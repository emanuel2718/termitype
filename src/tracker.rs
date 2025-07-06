use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::config::{Config, Mode};

const MAX_WPM_SAMPLES: usize = 300; // ~5 minutes at 1 sample per second

#[derive(Debug)]
pub struct Tracker {
    // metrics
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: u8,
    pub wpm_samples: Vec<u32>,
    pub last_sample_time: Instant,
    pub is_high_score: bool,

    // time
    pub time_started: Option<Instant>,
    pub time_end: Option<Instant>,
    pub time_paused: Option<Instant>,
    pub time_remaining: Option<Duration>,
    pub completion_time: Option<f64>,

    //  progress tracking
    pub user_input: Vec<Option<char>>,
    pub cursor_position: usize,
    pub target_text: String,
    pub target_chars: Vec<char>,
    pub word_count: usize,
    pub status: Status,
    pub total_keystrokes: usize,
    pub correct_keystrokes: usize,
    pub backspace_count: usize,
    pub wrong_words_start_indexes: HashSet<usize>,

    pub fps: FpsTracker,
    pub current_word_start: usize,
    pub last_metrics_update: Instant,

    // internal stufff
    current_word_is_correct_so_far: bool,
    max_user_input_length: usize,
    last_wpm_update: Instant,
    dirty_metrics: bool,
    cached_elapsed_seconds: f64,
    cached_elapsed_time: Instant,
    space_jump_stack: Vec<(usize, usize)>,
}

#[derive(Debug)]
pub struct FpsTracker {
    current_fps: f64,
    frame_count: u32,
    last_update: Instant,
    interval: Duration,
}

impl FpsTracker {
    fn new() -> Self {
        Self {
            current_fps: 0.0,
            frame_count: 0,
            last_update: Instant::now(),
            interval: Duration::from_millis(500),
        }
    }

    pub fn get(&self) -> f64 {
        self.current_fps
    }

    pub fn update(&mut self) -> bool {
        self.frame_count += 1;
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.interval {
            let elapsed = now.duration_since(self.last_update).as_secs_f64();
            self.current_fps = self.frame_count as f64 / elapsed;
            self.frame_count = 0;
            self.last_update = now;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Idle,
    Typing,
    Paused,
    Completed,
}

#[derive(Debug, PartialEq, Clone)]
struct TypingSpeed {
    wpm: f64,
    raw_wpm: f64,
}

impl Tracker {
    pub fn new(config: &Config, target_text: String) -> Self {
        let mode = config.current_mode();
        let word_count = mode.value();

        let target_length = target_text.len();
        let mut target_chars = Vec::with_capacity(target_length);
        target_chars.extend(target_text.chars());

        let time_remaining = match mode {
            Mode::Time { duration } => Some(Duration::from_secs(duration)),
            Mode::Words { .. } => None,
        };

        // try avoid re-allocations during typing at the cost of memory but worht it question mark
        let estimated_capacity = (target_length * 2).max(2048);

        let now = Instant::now();

        Self {
            wpm: 0.0,
            raw_wpm: 0.0,
            accuracy: 0,
            wpm_samples: Vec::with_capacity(100),
            last_sample_time: now,
            is_high_score: false,
            time_remaining,
            time_started: None,
            time_paused: None,
            time_end: None,
            completion_time: None,
            user_input: Vec::with_capacity(estimated_capacity),
            cursor_position: 0,
            target_text,
            target_chars,
            word_count,
            status: Status::Idle,
            total_keystrokes: 0,
            backspace_count: 0,
            correct_keystrokes: 0,
            wrong_words_start_indexes: HashSet::with_capacity(word_count / 3),
            current_word_start: 0,
            last_metrics_update: now,
            fps: FpsTracker::new(),
            current_word_is_correct_so_far: true,
            max_user_input_length: estimated_capacity,
            last_wpm_update: now,
            dirty_metrics: false,
            cached_elapsed_seconds: 0.0,
            cached_elapsed_time: now,
            space_jump_stack: Vec::new(),
        }
    }

    pub fn start_typing(&mut self) {
        let now = Instant::now();
        self.time_started = Some(now);
        self.last_sample_time = now;
        self.last_wpm_update = now;
        self.wpm_samples.clear();

        if let Some(duration) = self.time_remaining {
            let seconds = duration.as_secs();
            self.time_remaining = Some(Duration::from_secs(seconds));
            // NOTE(ema): must add a buffer of 500 to now have time jump from N to N-1 when test starts
            self.time_end = Some(now + Duration::from_secs(seconds) + Duration::from_millis(500));
        }

        self.wpm = 0.0;
        self.raw_wpm = 0.0;
        self.accuracy = 0;
        self.is_high_score = false;
        self.total_keystrokes = 0;
        self.correct_keystrokes = 0;
        self.completion_time = None;
        self.user_input.clear();
        self.cursor_position = 0;
        self.status = Status::Typing;
        self.wrong_words_start_indexes.clear();
        self.current_word_start = 0;
        self.current_word_is_correct_so_far = true;
        self.dirty_metrics = true;
        self.cached_elapsed_time = now;
        self.cached_elapsed_seconds = 0.0;
        self.space_jump_stack.clear();
    }

    // TODO: maybe mmove this elsewhere. not sure if it makes sense here.
    pub fn pause(&mut self) {
        if self.status == Status::Typing {
            self.status = Status::Paused;
            self.time_paused = Some(Instant::now());
        }
    }

    // TODO: maybe mmove this elsewhere. not sure if it makes sense here.
    pub fn resume(&mut self) {
        if self.status == Status::Paused {
            if let (Some(pause_start), Some(time_started)) = (self.time_paused, self.time_started) {
                let pause_duration = pause_start.elapsed();
                self.time_started = Some(time_started + pause_duration);

                if let Some(end_time) = self.time_end {
                    self.time_end = Some(end_time + pause_duration);
                }
            }
            self.status = Status::Typing;
            self.time_paused = None;
        }
    }

    #[inline(always)]
    pub fn type_char(&mut self, c: char) -> bool {
        if self.status == Status::Completed {
            return false;
        }

        if self.cursor_position >= self.target_chars.len() {
            return false;
        }

        if self.should_time_complete() {
            self.complete();
            return false;
        }

        if self.user_input.len() == self.user_input.capacity() {
            let new_capacity = self.user_input.capacity() * 2;
            self.user_input
                .reserve_exact(new_capacity - self.user_input.capacity());
            self.max_user_input_length = self.user_input.capacity();
        }

        // fast pass for correctly typed characters
        if self.status == Status::Typing
            && self.cursor_position < self.target_chars.len()
            && unsafe { *self.target_chars.get_unchecked(self.cursor_position) } == c
            && c != ' '
            && self.cursor_position != self.current_word_start
        {
            self.total_keystrokes += 1;
            self.correct_keystrokes += 1;

            self.user_input.push(Some(c));
            self.cursor_position += 1;
            self.dirty_metrics = true;

            // check for completion in `Word` mode to terminate test as soon as possible
            if self.time_remaining.is_none() && self.cursor_position >= self.target_chars.len() {
                self.check_completion();
            }

            return true;
        }

        let is_space = c == ' ';
        // don't allow space to EVER input a character on the first character of a word.
        // when was the last time you saw a `<space>` char be the first char of a word?
        //  wheere does a word start at?
        if is_space && self.cursor_position == self.current_word_start {
            return false;
        }

        // MAXIMUM PERFORMANCEEEEEEEEEEEEEEEEE!
        let target_char = unsafe { *self.target_chars.get_unchecked(self.cursor_position) };
        let is_correct = target_char == c;

        // mimic space jump logic from monketype
        if is_space && target_char != ' ' {
            self.wrong_words_start_indexes
                .insert(self.current_word_start);

            let mut next_space_pos = self.cursor_position;
            while next_space_pos < self.target_chars.len()
                && self.target_chars[next_space_pos] != ' '
            {
                next_space_pos += 1;
            }

            if next_space_pos < self.target_chars.len() {
                let target_pos = next_space_pos + 1;

                self.space_jump_stack
                    .push((self.cursor_position, target_pos));

                self.cursor_position = target_pos;
                self.current_word_start = self.cursor_position;
                self.current_word_is_correct_so_far = true;

                while self.user_input.len() < self.cursor_position {
                    self.user_input.push(None);
                }

                return true;
            }
        }

        if !is_correct && target_char == ' ' {
            self.register_keystroke(false);
            return true;
        }

        self.register_keystroke(is_correct);

        self.user_input.push(Some(c));

        // word-correctness check
        let was_correct = self.current_word_is_correct_so_far;
        self.current_word_is_correct_so_far = was_correct & is_correct;

        if !self.current_word_is_correct_so_far & was_correct {
            self.wrong_words_start_indexes
                .insert(self.current_word_start);
        }

        self.cursor_position += 1;

        // boundary detection
        if is_space && target_char == ' ' {
            if self
                .wrong_words_start_indexes
                .contains(&self.current_word_start)
            {
                self.validate_completed_word(self.current_word_start, self.cursor_position - 1);
            }
            self.current_word_start = self.cursor_position;
            self.current_word_is_correct_so_far = true;
        }

        // check for completion in `Word` mode
        if self.time_remaining.is_none() && self.cursor_position >= self.target_chars.len() {
            self.check_completion();
        }

        self.dirty_metrics = true;
        true
    }

    #[inline(always)]
    fn register_keystroke(&mut self, is_correct: bool) {
        self.total_keystrokes += 1;
        self.correct_keystrokes += is_correct as usize;
    }

    pub fn needs_metrics_update(&self) -> bool {
        self.dirty_metrics && self.status == Status::Typing
    }

    pub fn update_metrics(&mut self) {
        if self.status == Status::Completed || self.status == Status::Paused {
            return;
        }

        let Some(start_time) = self.time_started else {
            return;
        };

        let now = Instant::now();

        if let Some(end_time) = self.time_end {
            self.time_remaining = Some(end_time.duration_since(now));
        }

        self.accuracy = if self.total_keystrokes > 0 {
            ((self.correct_keystrokes * 100) / self.total_keystrokes).min(100) as u8
        } else {
            0
        };

        // limit the times we update the wpm to avoid jittery wpm updates
        let should_update_wpm = now.duration_since(self.last_wpm_update) >= Duration::from_secs(1);
        if should_update_wpm {
            if let Some(speed) = self.calculate_typing_speed(start_time) {
                self.wpm = speed.wpm;
                self.raw_wpm = speed.raw_wpm;
                self.update_wpm_samples(speed.wpm, false);
                self.last_wpm_update = now;
            }
        }

        self.dirty_metrics = false;
    }

    pub fn backspace(&mut self) -> bool {
        if self.status == Status::Completed {
            return false;
        }

        if self.cursor_position == 0 {
            return false;
        }

        // backward space jump check
        if let Some(&(prev_source_pos, prev_target_pos)) = self.space_jump_stack.last() {
            if self.cursor_position == prev_target_pos {
                self.cursor_position = prev_source_pos;
                self.user_input.truncate(self.cursor_position);
                self.space_jump_stack.pop();
                self.current_word_start = 0;

                for i in (0..self.cursor_position).rev() {
                    if i < self.user_input.len() && self.user_input[i] == Some(' ') {
                        self.current_word_start = i + 1;
                        break;
                    }
                }

                self.backspace_count += 1;
                return true;
            }
        }

        self.backspace_count += 1;

        // just typed a space
        let at_word_boundary = self.cursor_position > 0
            && self.cursor_position <= self.user_input.len()
            && self.user_input.get(self.cursor_position - 1) == Some(&Some(' '));

        if at_word_boundary {
            // start of a new word - only allow backspace if previous word was wrong
            let prev_word_start_idx = self.get_previous_word_start();
            let is_prev_word_wrong = self.is_word_wrong(prev_word_start_idx);

            if !is_prev_word_wrong {
                return false;
            }

            self.current_word_start = prev_word_start_idx;
        }

        let mut word_start = 0;
        for i in (0..self.cursor_position).rev() {
            if i < self.user_input.len() && self.user_input[i] == Some(' ') {
                word_start = i + 1;
                break;
            }
        }

        // reset word correctness on backspace as we need to type at least (1) more char
        // to reach the word boundary (where we do the correctness validation) after a backspace.
        self.wrong_words_start_indexes.remove(&word_start);
        self.current_word_is_correct_so_far = true;

        // do the actual backspace
        self.user_input.pop();
        self.cursor_position -= 1;

        true
    }

    /// Calculates typing speed based on elapsed time
    fn calculate_speed_from_duration(&self, elapsed: f64, ensure_min: bool) -> Option<TypingSpeed> {
        // avoids jittery WPM
        if ensure_min && elapsed < 0.5 {
            return None;
        }

        // no division by 0
        let elapsed_seconds = elapsed.max(0.1);
        let elapsed_minutes = elapsed_seconds / 60.0;

        let correct_words = (self.correct_keystrokes as f64) * 0.2;
        let total_chars = self.user_input.len() as f64;
        let total_words = total_chars * 0.2;

        Some(TypingSpeed {
            wpm: (correct_words / elapsed_minutes).max(0.0),
            raw_wpm: (total_words / elapsed_minutes).max(0.0),
        })
    }

    /// Calculates current typing speed for live updates
    fn calculate_typing_speed(&self, start_time: Instant) -> Option<TypingSpeed> {
        let elapsed_seconds = start_time.elapsed().as_secs_f64();
        self.calculate_speed_from_duration(elapsed_seconds, true)
    }

    /// Sets final WPM metrics when test completes
    fn set_final_wpm(&mut self, completion_time: f64) {
        if let Some(final_speed) = self.calculate_speed_from_duration(completion_time, false) {
            self.wpm = final_speed.wpm;
            self.raw_wpm = final_speed.raw_wpm;
            self.update_wpm_samples(final_speed.wpm, true);
        }
    }

    /// Calculate typing consistency as a percentage up to (2) decimal places.
    pub fn calculate_consistency(&self) -> f64 {
        if self.wpm_samples.len() < 2 {
            return 100.0;
        }

        let samples: Vec<f64> = self.wpm_samples.iter().map(|&x| x as f64).collect();
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;

        // no division by 0 on my watch
        if mean < 1.0 {
            return 0.0;
        }

        let variance =
            samples.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (samples.len() - 1) as f64;
        let std_dev = variance.sqrt();

        let cv = std_dev / mean; // coefficient of variation

        let consistency = (1.0 - cv.clamp(0.0, 1.0)) * 100.0;

        (consistency * 100.0).round() / 100.0
    }

    pub fn update_wpm_samples(&mut self, wpm: f64, force: bool) {
        let now = Instant::now();
        if force || now.duration_since(self.last_wpm_update) >= Duration::from_secs(1) {
            if self.wpm_samples.len() >= MAX_WPM_SAMPLES {
                let remove_count = MAX_WPM_SAMPLES / 10;
                self.wpm_samples.drain(0..remove_count);
            }
            self.wpm_samples.push(wpm.round() as u32);
            self.last_wpm_update = now;
        }
    }

    pub fn mark_high_score(&mut self) {
        self.is_high_score = true;
    }

    fn check_completion(&mut self) -> bool {
        if self.status != Status::Typing {
            return false;
        }
        let is_complete = match self.time_remaining {
            Some(rem) if rem.as_secs() == 0 => true,
            None => self.cursor_position >= self.target_chars.len(),
            _ => false,
        };
        if is_complete {
            let completion_time = self
                .time_started
                .map(|start| start.elapsed().as_secs_f64())
                .unwrap_or(0.0);
            self.completion_time = Some(completion_time);

            self.set_final_wpm(completion_time);

            self.status = Status::Completed;
        }
        is_complete
    }

    pub fn complete(&mut self) {
        let start_time = self.time_started.unwrap_or(Instant::now());
        let end_time = self.time_end.unwrap_or(Instant::now());
        self.time_remaining = Some(Duration::from_secs(0));
        let completion_time = end_time.duration_since(start_time).as_secs_f64();
        self.completion_time = Some(completion_time);

        self.set_final_wpm(completion_time);

        self.status = Status::Completed;
    }

    /// Returns the start index of the previous word.
    fn get_previous_word_start(&self) -> usize {
        let mut pos = self.cursor_position - 1; // Start from before the space
        while pos > 0 && self.target_chars.get(pos - 1) != Some(&' ') {
            pos -= 1;
        }
        pos
    }

    /// Returns true if the word at the given start index is marked as wrong.
    pub fn is_word_wrong(&self, start_idx: usize) -> bool {
        self.wrong_words_start_indexes.contains(&start_idx)
    }

    /// Validates the last complete typed word and marks the word as correct/incorrect
    #[inline(always)]
    fn validate_completed_word(&mut self, word_start: usize, word_end: usize) {
        let mut target_end = word_start;
        while target_end < self.target_chars.len() && self.target_chars[target_end] != ' ' {
            target_end += 1;
        }

        let user_word_len = word_end - word_start;
        let target_word_len = target_end - word_start;

        if user_word_len != target_word_len {
            return;
        }

        // check if any of the chars don't match between the current word and what was typed.
        for i in word_start..word_end {
            let user_char = self.user_input.get(i).and_then(|c| *c);
            let target_char = self.target_chars.get(i).copied();

            if user_char != target_char {
                return;
            }
        }

        self.wrong_words_start_indexes.remove(&word_start);
    }

    /// Checks if the test has reached its conclusion. Only applicable in Time mode
    pub fn should_time_complete(&self) -> bool {
        if self.status != Status::Typing {
            return false;
        }
        if let Some(end_time) = self.time_end {
            Instant::now() >= end_time
        } else {
            false
        }
    }

    /// Smartly updates the time remaining for `Time` mode.
    pub fn update_time_remaining(&mut self) -> bool {
        if self.status != Status::Typing {
            return false;
        }
        // this means that we are in time mode...but we should have a better way to determine this
        if let Some(end_time) = self.time_end {
            let new_rem = end_time.saturating_duration_since(Instant::now());
            let curr_seconds = self.time_remaining.map(|t| t.as_secs()).unwrap_or(0);
            let new_seconds = new_rem.as_secs();
            // only if the seconds differ we update the time remaining
            if curr_seconds != new_seconds {
                self.time_remaining = Some(new_rem);
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_tracker() -> Tracker {
        let config = Config::default();
        let target_text = String::from("hello world");
        Tracker::new(&config, target_text)
    }

    #[test]
    fn test_default_state() {
        let tracker = create_tracker();
        assert_eq!(tracker.status, Status::Idle);
        assert_eq!(tracker.wpm, 0.0);
        assert_eq!(tracker.raw_wpm, 0.0);
        assert_eq!(tracker.accuracy, 0);
        assert_eq!(tracker.cursor_position, 0);
        assert!(tracker.time_started.is_none());
        assert!(tracker.completion_time.is_none());
    }

    #[test]
    fn test_backscpace_count_tracking() {
        let mut tracker = create_tracker();
        tracker.backspace();
        assert_eq!(tracker.backspace_count, 0);

        tracker.start_typing();
        tracker.backspace();
        assert_eq!(tracker.backspace_count, 0);

        for i in 0..10 {
            tracker.type_char(char::from(i));
            tracker.backspace();
        }
        assert_eq!(tracker.backspace_count, 10);
    }

    #[test]
    fn test_typing_lifecycle() {
        let mut tracker = create_tracker();

        tracker.start_typing();
        assert_eq!(tracker.status, Status::Typing);
        assert!(tracker.time_started.is_some());

        assert!(tracker.type_char('h'));
        // NOTE: we need to artifially call `update_metrics` as it is only called on every tick.
        tracker.update_metrics();
        assert_eq!(tracker.accuracy, 100);
        assert_eq!(tracker.cursor_position, 1);

        assert!(tracker.type_char('x'));
        tracker.update_metrics();
        assert_eq!(tracker.accuracy, 50);
        assert_eq!(tracker.cursor_position, 2);

        assert!(tracker.backspace());
        assert_eq!(tracker.cursor_position, 1);

        assert_eq!(tracker.correct_keystrokes, 1);
    }

    #[test]
    fn test_typing_wrong_char_on_word_boundary_should_do_nothing() {
        let config = Config::default();
        let target_text = String::from("xyz hello");
        let mut tracker = Tracker::new(&config, target_text);

        tracker.start_typing();

        assert!(tracker.type_char('x'));
        assert!(tracker.type_char('y'));
        assert!(tracker.type_char('z'));
        assert!(tracker.type_char('x'));

        assert_eq!(tracker.cursor_position, 3);
        assert_eq!(tracker.correct_keystrokes, 3);

        assert!(tracker.type_char('y'));

        assert_eq!(tracker.cursor_position, 3);
        assert_eq!(tracker.correct_keystrokes, 3);
    }

    #[test]
    fn test_pause_resume() {
        let mut tracker = create_tracker();

        tracker.start_typing();
        assert_eq!(tracker.status, Status::Typing);

        tracker.pause();
        assert_eq!(tracker.status, Status::Paused);
        assert!(tracker.time_paused.is_some());

        tracker.resume();
        assert_eq!(tracker.status, Status::Typing);
        assert!(tracker.time_paused.is_none());
    }

    #[test]
    fn test_wrong_word_backspace() {
        let mut tracker = create_tracker();
        tracker.start_typing();

        // Type "hallo" (wrong) instead of "hello"
        tracker.type_char('h');
        tracker.type_char('a');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');
        assert!(tracker.is_word_wrong(0));

        // backtrack
        for _ in 0.."hallo".len() {
            tracker.backspace();
        }
        assert!(!tracker.is_word_wrong(0));

        // Type "hello" correctly
        tracker.type_char('h');
        tracker.type_char('e');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');
        assert!(!tracker.is_word_wrong(0));
    }

    #[test]
    fn test_multiple_wrong_words() {
        let config = Config::default();
        let target_text = String::from("hello world pog");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        // Type "hallo world pa"
        // First word wrong
        tracker.type_char('h');
        tracker.type_char('a');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');
        tracker.type_char(' ');

        // Second word correct
        tracker.type_char('w');
        tracker.type_char('o');
        tracker.type_char('r');
        tracker.type_char('l');
        tracker.type_char('d');
        tracker.type_char(' ');

        // Third word wrong
        tracker.type_char('p');
        tracker.type_char('a');

        assert!(!tracker.is_word_wrong(6));
        assert!(tracker.is_word_wrong(12));

        // is first word still marked as wrong? should be
        assert!(tracker.is_word_wrong(0));
    }

    #[test]
    fn test_only_add_to_wrong_words_when_jumping_to_next_word() {
        let config = Config::default();
        let target_text = String::from("hello world pog");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        tracker.type_char('h');
        assert!(tracker.wrong_words_start_indexes.is_empty());
        tracker.type_char(' ');
        assert!(tracker.is_word_wrong(0));
        assert_eq!(tracker.wrong_words_start_indexes.len(), 1);
        tracker.type_char(' ');
        // NOTE: before, it would keep adding to `wrong_words_start_indexes` on every <space>
        //       if the wrong char was typed
        assert_eq!(tracker.wrong_words_start_indexes.len(), 1);
    }

    #[test]
    fn test_time_based_completion() {
        let config = Config::default();
        let target_text = String::from("test text for timing");
        let mut tracker = Tracker::new(&config, target_text);

        tracker.time_remaining = Some(Duration::from_secs(1));
        tracker.start_typing();

        assert!(tracker.type_char('t'));
        assert!(tracker.type_char('e'));
        assert_eq!(tracker.status, Status::Typing);

        tracker.time_end = Some(Instant::now());

        assert!(!tracker.type_char('s'));
        assert_eq!(tracker.status, Status::Completed);
        assert_eq!(tracker.time_remaining, Some(Duration::from_secs(0)));
    }

    #[test]
    fn test_completion_time_accuracy() {
        let config = Config::default();
        let target_text = String::from("test");
        let mut tracker = Tracker::new(&config, target_text);

        tracker.time_remaining = Some(Duration::from_secs(1));
        tracker.start_typing();

        tracker.complete();

        assert_eq!(tracker.completion_time, Some(1.5));
        assert_eq!(tracker.status, Status::Completed);
    }

    #[test]
    fn test_accented_characters() {
        let config = Config::default();
        let target_text = String::from("café résumé");
        let mut tracker = Tracker::new(&config, target_text);

        tracker.start_typing();

        // Type "café"
        assert!(tracker.type_char('c'));
        tracker.update_metrics();
        assert!(tracker.type_char('a'));
        tracker.update_metrics();
        assert!(tracker.type_char('f'));
        tracker.update_metrics();
        assert!(tracker.type_char('é'));
        tracker.update_metrics();
        assert!(tracker.type_char(' '));
        tracker.update_metrics();

        // Type "résumé"
        assert!(tracker.type_char('r'));
        tracker.update_metrics();
        assert!(tracker.type_char('é'));
        tracker.update_metrics();
        assert!(tracker.type_char('s'));
        tracker.update_metrics();
        assert!(tracker.type_char('u'));
        tracker.update_metrics();
        assert!(tracker.type_char('m'));
        tracker.update_metrics();
        assert!(tracker.type_char('é'));
        tracker.update_metrics();

        assert_eq!(tracker.correct_keystrokes, 11);
        assert_eq!(tracker.total_keystrokes, 11);
        assert_eq!(tracker.cursor_position, 11);
        assert!(tracker.wrong_words_start_indexes.is_empty());
    }

    #[test]
    fn test_accented_characters_with_wrong_input() {
        let config = Config::default();
        let target_text = String::from("café");
        let mut tracker = Tracker::new(&config, target_text);

        tracker.start_typing();

        // Type "cafe" (without accent)
        assert!(tracker.type_char('c'));
        tracker.update_metrics();
        assert!(tracker.type_char('a'));
        tracker.update_metrics();
        assert!(tracker.type_char('f'));
        tracker.update_metrics();
        assert!(tracker.type_char('e')); // wrong: 'e' instead of 'é'
        tracker.update_metrics();

        assert_eq!(tracker.correct_keystrokes, 3);
        assert_eq!(tracker.total_keystrokes, 4);
        assert_eq!(tracker.cursor_position, 4);
        assert!(!tracker.wrong_words_start_indexes.is_empty());
    }

    #[test]
    fn test_accented_characters_backspace() {
        let config = Config::default();
        let target_text = String::from("café résumé");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.time_remaining = Some(Duration::from_secs(30));
        tracker.start_typing();
        tracker.time_end = Some(Instant::now() + Duration::from_secs(30));

        // Type "café" correctly
        assert!(tracker.type_char('c'));
        tracker.update_metrics();
        assert!(tracker.type_char('a'));
        tracker.update_metrics();
        assert!(tracker.type_char('f'));
        tracker.update_metrics();
        assert!(tracker.type_char('é'));
        tracker.update_metrics();

        // Backspace the é
        assert!(tracker.backspace());
        tracker.update_metrics();
        assert_eq!(tracker.cursor_position, 3);

        // Type wrong character 'e'
        assert!(tracker.type_char('e'));
        tracker.update_metrics();
        assert!(tracker.is_word_wrong(0));

        // Backspace and fix
        assert!(tracker.backspace());
        tracker.update_metrics();
        assert!(tracker.type_char('é'));
        tracker.update_metrics();
        assert!(!tracker.is_word_wrong(0));

        // Complete the word
        assert!(tracker.type_char(' '));
        tracker.update_metrics();

        // Verify the state
        assert_eq!(tracker.cursor_position, 5);
        assert_eq!(tracker.total_keystrokes, 7); // Including the wrong 'e' and backspace
    }

    #[test]
    fn test_disallow_backspace_at_word_boundary() {
        let config = Config::default();
        let target_text = String::from("hello world");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        tracker.type_char('h');
        tracker.type_char('e');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');
        tracker.type_char(' ');

        assert!(
            !tracker.backspace(),
            "Should not allow backspace after correct word"
        );
        assert_eq!(
            tracker.cursor_position, 6,
            "Cursor position should not change"
        );
        assert_eq!(
            tracker.user_input.len(),
            6,
            "Input length should not change"
        );
    }

    #[test]
    fn test_allow_backspace_at_word_boundary() {
        let config = Config::default();
        let target_text = String::from("hello world");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        tracker.type_char('h');
        tracker.type_char('e');
        tracker.type_char('y');
        tracker.type_char('y');
        tracker.type_char('o');
        tracker.type_char(' ');

        // Should allow backspace after incorrect word
        assert!(
            tracker.backspace(),
            "Should allow backspace after incorrect word"
        );
        assert_eq!(
            tracker.cursor_position, 5,
            "Cursor position should decrease"
        );
        assert_eq!(tracker.user_input.len(), 5, "Input length should decrease");
    }

    #[test]
    fn test_correcting_wrong_word_without_backspace() {
        let config = Config::default();
        let target_text = String::from("hello world");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        tracker.type_char('h');
        tracker.type_char('a');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');

        assert!(tracker.is_word_wrong(0), "Word should be marked as wrong");

        for _ in 0..5 {
            tracker.backspace();
        }

        tracker.type_char('h');
        tracker.type_char('e');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');

        assert!(
            !tracker.is_word_wrong(0),
            "Word should not be marked as wrong after correction"
        );
    }

    #[test]
    fn test_in_place_correction_without_backspace() {
        let config = Config::default();
        let target_text = String::from("abc");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        tracker.type_char('a');
        tracker.type_char('x');
        assert!(tracker.is_word_wrong(0), "Word should be marked as wrong");

        tracker.backspace();

        tracker.type_char('b');
        tracker.type_char('c');

        assert!(
            !tracker.is_word_wrong(0),
            "Word should be unmarked when fixed"
        );
    }

    #[test]
    fn test_fix_previous_word() {
        let config = Config::default();
        let target_text = String::from("hello world test");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        tracker.type_char('h');
        tracker.type_char('a');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');
        tracker.type_char(' ');

        assert!(
            tracker.is_word_wrong(0),
            "First word should be marked as wrong"
        );

        tracker.type_char('w');
        tracker.type_char('o');
        tracker.type_char('r');
        tracker.type_char('l');
        tracker.type_char('d');
        tracker.type_char(' ');
        tracker.type_char('t');

        for _ in 0..8 {
            tracker.backspace();
        }

        // Now let's manually fix the word
        // Force current_word_start to be at the beginning of the first word
        tracker.current_word_start = 0;

        // Type the correct character
        tracker.type_char('h');
        tracker.type_char('e');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');

        // Force the removal of the first word from wrong_words
        tracker.wrong_words_start_indexes.remove(&0);

        // Continue
        tracker.type_char(' ');

        // Check if it worked
        assert!(
            !tracker.is_word_wrong(0),
            "First word should no longer be marked wrong"
        );
    }

    #[test]
    fn test_fix_word_and_unmark() {
        let config = Config::default();
        let target_text = String::from("hello world");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        tracker.type_char('h');
        tracker.type_char('a'); // wrong
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');

        assert!(tracker.is_word_wrong(0), "Word should be marked as wrong");

        tracker.backspace(); // 'o'
        tracker.backspace(); // 'l'
        tracker.backspace(); // 'l'
        tracker.backspace(); // 'a'
        tracker.backspace(); // 'h'

        assert!(
            !tracker.is_word_wrong(0),
            "Word should not be in wrong set after backspacing"
        );

        tracker.type_char('h');
        tracker.type_char('e');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');

        assert!(
            !tracker.is_word_wrong(0),
            "Word should not be marked wrong when typed correctly"
        );

        tracker.type_char(' ');

        assert!(
            !tracker.is_word_wrong(0),
            "Word should still not be in wrong set after completing it"
        );
    }

    #[test]
    fn test_partial_wrong_word_fix() {
        let config = Config::default();
        let target_text = String::from("hello world");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();
        tracker.type_char('f');
        tracker.type_char('f');
        tracker.type_char('f');
        tracker.type_char('f');
        tracker.type_char('f');

        tracker.backspace();
        tracker.backspace();
        tracker.type_char('l');
        tracker.type_char('0');
        tracker.type_char(' ');
        assert!(tracker.is_word_wrong(0), "Word should be marked wrong");
    }

    #[test]
    fn test_backspace_at_word_boundary_behavior() {
        let config = Config::default();
        let target_text = String::from("hello world");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        tracker.type_char('h');
        tracker.type_char('e');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');
        tracker.type_char(' ');

        assert!(
            !tracker.backspace(),
            "Should not allow backspace after correct word"
        );
        assert_eq!(
            tracker.cursor_position, 6,
            "Cursor should remain after space"
        );

        tracker.type_char('x'); // wrong (should be 'w')
        assert!(
            tracker.backspace(),
            "Should allow backspace after incorrect character"
        );

        tracker.type_char('w');
        tracker.type_char('o');
        tracker.type_char('r');
        tracker.type_char('l');
        tracker.type_char('d');

        assert!(
            tracker.wrong_words_start_indexes.is_empty(),
            "Should have no wrong words"
        );
    }

    #[test]
    fn test_consistency_calculation() {
        let mut tracker = create_tracker();

        assert_eq!(tracker.calculate_consistency(), 100.0);

        tracker.wpm_samples = vec![50];
        assert_eq!(tracker.calculate_consistency(), 100.0);

        tracker.wpm_samples = vec![50, 50, 50];
        assert_eq!(tracker.calculate_consistency(), 100.0);

        // mean=50, std_dev=5 => variation would be 10% ==> consistentcy is 90%
        tracker.wpm_samples = vec![45, 50, 55];
        let consistency = tracker.calculate_consistency();
        assert!(
            (89.0..91.0).contains(&consistency),
            "Expected consistency around 90% for 10% variation, got {consistency}%",
        );

        // mean=50, std_dev=25 => variation would be 50% ==> consistentcy is 50%
        tracker.wpm_samples = vec![25, 50, 75];
        let consistency = tracker.calculate_consistency();
        assert!(
            (49.0..51.0).contains(&consistency),
            "Expected consistency around 50% for 50% variation, got {consistency}%",
        );
    }

    #[test]
    fn test_wpm_sample_collection_during_typing() {
        let mut tracker = create_tracker();
        tracker.start_typing();

        for c in "hello".chars() {
            tracker.type_char(c);
        }

        assert!(tracker.wpm_samples.is_empty());

        // Ensure at least 0.5 seconds have elapsed for calculate_typing_speed
        tracker.time_started = Some(Instant::now() - Duration::from_millis(600));
        if let Some(speed) = tracker.calculate_typing_speed(tracker.time_started.unwrap()) {
            tracker.update_wpm_samples(speed.wpm, true); // Force collection
        }
        assert_eq!(tracker.wpm_samples.len(), 1);

        for c in " world".chars() {
            tracker.type_char(c);
        }

        // Force another sample collection
        tracker.time_started = Some(Instant::now() - Duration::from_millis(700));
        if let Some(speed) = tracker.calculate_typing_speed(tracker.time_started.unwrap()) {
            tracker.update_wpm_samples(speed.wpm, true); // Force collection
        }
        assert_eq!(tracker.wpm_samples.len(), 2);

        assert!(tracker.wpm_samples[0] != tracker.wpm_samples[1]);
    }

    #[test]
    fn test_wpm_sample_collection() {
        let mut tracker = create_tracker();
        tracker.start_typing();

        for c in "hello world".chars() {
            tracker.type_char(c);
        }

        // Ensure at least 0.5 seconds have elapsed for calculate_typing_speed
        tracker.time_started = Some(Instant::now() - Duration::from_millis(600));
        if let Some(speed) = tracker.calculate_typing_speed(tracker.time_started.unwrap()) {
            tracker.update_wpm_samples(speed.wpm, true); // Force collection
        }
        assert_eq!(
            tracker.wpm_samples.len(),
            1,
            "Should have collected first sample"
        );

        // Force another sample collection
        tracker.time_started = Some(Instant::now() - Duration::from_millis(700));
        if let Some(speed) = tracker.calculate_typing_speed(tracker.time_started.unwrap()) {
            tracker.update_wpm_samples(speed.wpm, true); // Force collection
        }
        assert_eq!(
            tracker.wpm_samples.len(),
            2,
            "Should collect new sample after interval"
        );
    }

    #[test]
    fn test_space_at_start_of_test() {
        let config = Config::default();
        let target_text = String::from("hello termitype");
        let mut tracker = Tracker::new(&config, target_text);

        assert!(!tracker.type_char(' '));
        assert_eq!(tracker.status, Status::Idle);
        assert_eq!(tracker.cursor_position, 0);
        assert!(tracker.user_input.is_empty());

        assert!(tracker.type_char('h'));
        assert_eq!(tracker.cursor_position, 1);
        assert_eq!(tracker.user_input.len(), 1);
    }

    #[test]
    fn test_space_at_beginning_of_word() {
        let config = Config::default();
        let target_text = String::from("hello termitype");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        assert!(tracker.type_char('h'));
        assert!(tracker.type_char('e'));
        assert!(tracker.type_char('l'));
        assert!(tracker.type_char('l'));
        assert!(tracker.type_char('o'));
        assert!(tracker.type_char(' '));

        assert_eq!(tracker.current_word_start, 6);

        assert!(!tracker.type_char(' '));
        assert_eq!(tracker.cursor_position, 6);
        assert_eq!(tracker.user_input.len(), 6);

        assert!(tracker.type_char('t'));
        assert_eq!(tracker.cursor_position, 7);
        assert_eq!(tracker.user_input.len(), 7);
    }

    #[test]
    fn test_mistype_and_fix_word_should_unmark_as_wrong() {
        // scenario taken from an actual live reproduction of the bug
        // `termitype --words "about too soon"`
        let config = Config::default();
        let target_text = String::from("about too soon");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();

        assert!(tracker.type_char('a')); // cursor = 1
        assert!(tracker.type_char('b')); // cursor = 2
        assert!(tracker.type_char('o')); // cursor = 3
        assert!(tracker.type_char('u')); // cursor = 4
        assert!(tracker.type_char('t')); // cursor = 5
        assert!(tracker.type_char(' ')); // cursor = 6

        // 'too' starts at 6 in this case
        assert_eq!(tracker.current_word_start, 6);
        assert!(tracker.wrong_words_start_indexes.is_empty());

        // type 'too' but make error by hitting space when `o` is expected
        assert!(tracker.type_char('t')); // cursor = 7
        assert!(tracker.type_char('o')); // cursor = 8
        assert!(tracker.type_char(' ')); // cursor = 9, wrong char

        assert!(
            tracker.is_word_wrong(6),
            "Word 'too' should be marked wrong after early space"
        );

        // fix mistake
        assert!(tracker.backspace()); // cursor = 8

        // complete the wrod
        assert!(tracker.type_char('o')); // cursor = 9
        assert!(tracker.type_char(' ')); // cursor = 10

        assert!(
            !tracker.is_word_wrong(6),
            "Word 'too' should NOT be marked wrong after correction. Wrong words: {:?}",
            tracker.wrong_words_start_indexes
        );
    }

    #[test]
    fn test_space_beyond_first_letter_should_skip_to_next_word() {
        let mut tracker = create_tracker(); // will be `hello world`
        tracker.start_typing();
        tracker.type_char(' ');
        assert_eq!(tracker.cursor_position, 0);
        tracker.type_char('h');
        assert_eq!(tracker.cursor_position, 1);
        tracker.type_char(' ');
        assert_eq!(tracker.cursor_position, 6);
    }

    #[test]
    fn test_space_in_boundary_shouldnt_skip() {
        let mut tracker = create_tracker(); // will be `hello world`
        tracker.start_typing();
        for c in "hello".chars() {
            tracker.type_char(c);
        }
        assert_eq!(tracker.cursor_position, 5);
        tracker.type_char(' ');
        assert_eq!(tracker.cursor_position, 6);
        tracker.type_char(' ');
        assert_eq!(tracker.cursor_position, 6);
    }

    #[test]
    fn test_backspace_after_space_beyond_first_letter() {
        let mut tracker = create_tracker(); // will be `hello world`
        tracker.start_typing();
        tracker.type_char('h');
        assert_eq!(tracker.cursor_position, 1);
        tracker.type_char(' ');
        assert_eq!(tracker.cursor_position, 6);
        // should take you back after a space jump
        tracker.backspace();
        assert_eq!(tracker.cursor_position, 1);
    }

    #[test]
    fn test_backspace_after_multiple_chained_spaces() {
        let config = Config::default();
        let target_text = String::from("hello there termitype hope you doing good");
        let mut tracker = Tracker::new(&config, target_text);
        tracker.start_typing();
        tracker.type_char('h');
        assert_eq!(tracker.cursor_position, 1);
        tracker.type_char(' ');
        assert_eq!(tracker.cursor_position, 6); // just before "there"
        tracker.type_char('t');
        assert_eq!(tracker.cursor_position, 7);
        tracker.type_char(' ');
        assert_eq!(tracker.cursor_position, 12); // just before "termitype"
        tracker.backspace();
        assert_eq!(tracker.cursor_position, 7);
        tracker.backspace();
        assert_eq!(tracker.cursor_position, 6);
        tracker.backspace();
        assert_eq!(tracker.cursor_position, 1);
    }

    #[test]
    fn test_word_mode_completion() {
        let config = Config::default();
        let target_text = String::from("hello world test end");
        let mut tracker = Tracker::new(&config, target_text);

        tracker.time_remaining = None;
        tracker.start_typing();

        for c in "hello world test end".chars() {
            assert!(tracker.type_char(c), "Should accept character '{c}'");

            if tracker.cursor_position >= tracker.target_chars.len() {
                assert_eq!(
                    tracker.status,
                    Status::Completed,
                    "Test should be completed when all characters are typed"
                );
                break;
            }
        }

        assert_eq!(
            tracker.status,
            Status::Completed,
            "Test should be completed after typing all words"
        );
        assert_eq!(
            tracker.cursor_position,
            tracker.target_chars.len(),
            "Cursor should be at end of text"
        );
        assert!(
            tracker.completion_time.is_some(),
            "Completion time should be set"
        );
    }

    #[test]
    fn test_restarting_test_resets_high_score_flag() {
        let config = Config::default();
        let target_text = String::from("test");
        let mut tracker = Tracker::new(&config, target_text);
        assert!(!tracker.is_high_score);
        tracker.start_typing();

        tracker.is_high_score = true;

        assert!(tracker.is_high_score);

        tracker.start_typing();
        assert!(!tracker.is_high_score);
    }

    #[test]
    fn test_fast_words_mode_wpm_calculation() {
        let mut config = Config::default();
        config.time = None;
        config.word_count = Some(1);
        let target_text = String::from("test");
        let mut tracker = Tracker::new(&config, target_text);

        tracker.start_typing();

        tracker.time_started = Some(Instant::now() - Duration::from_millis(100)); // 0.1 seconds ago

        for c in "test".chars() {
            tracker.type_char(c);
        }

        assert_eq!(tracker.status, Status::Completed);
        assert!(tracker.wpm > 0.0,);
        assert!(tracker.raw_wpm > 0.0,);

        // 4 chars = 0.8 words, in 0.1 seconds = 0.8 / (0.1/60) = 480 WPM
        assert!(tracker.wpm > 400.0);
    }
}
