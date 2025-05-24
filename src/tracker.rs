use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::config::{Config, Mode};

#[derive(Debug)]
pub struct Tracker {
    // metrics
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: u8,
    pub wpm_samples: Vec<u32>,
    pub last_sample_time: Instant,

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

    pub current_word_start: usize,
    pub last_metrics_update: Instant,

    // Smart word tracking for performance
    current_word_is_correct_so_far: bool,

    max_user_input_length: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Idle,
    Typing,
    Paused,
    Completed,
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

        Self {
            wpm: 0.0,
            raw_wpm: 0.0,
            accuracy: 0,
            wpm_samples: Vec::with_capacity(100), // Pre-allocate space for samples
            last_sample_time: Instant::now(),
            time_remaining,
            time_started: None,
            time_paused: None,
            time_end: None,
            completion_time: None,
            user_input: Vec::with_capacity(target_length),
            cursor_position: 0,
            target_text,
            target_chars,
            word_count,
            status: Status::Idle,
            total_keystrokes: 0,
            backspace_count: 0,
            correct_keystrokes: 0,
            wrong_words_start_indexes: HashSet::with_capacity(word_count / 5), // guesstimate
            current_word_start: 0,
            last_metrics_update: Instant::now(),
            current_word_is_correct_so_far: true,
            max_user_input_length: target_length,
        }
    }

    pub fn start_typing(&mut self) {
        let now = Instant::now();
        self.time_started = Some(now);
        self.last_sample_time = now;
        self.wpm_samples.clear();

        if let Some(duration) = self.time_remaining {
            let seconds = duration.as_secs();
            self.time_remaining = Some(Duration::from_secs(seconds));
            self.time_end = Some(now + Duration::from_secs(seconds));
        }

        self.wpm = 0.0;
        self.raw_wpm = 0.0;
        self.accuracy = 0;
        self.total_keystrokes = 0;
        self.correct_keystrokes = 0;
        self.completion_time = None;
        self.user_input.clear();
        self.cursor_position = 0;
        self.status = Status::Typing;
        self.wrong_words_start_indexes.clear();
        self.current_word_start = 0;
        self.current_word_is_correct_so_far = true;
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

    pub fn type_char(&mut self, c: char) -> bool {
        if self.status == Status::Completed {
            return false;
        }

        // compatibility with that monkey famous game we are simulating...
        // first char is <space> and we are at the start of a word? Do nothing
        let is_space = c == ' ';
        if is_space && self.cursor_position == self.current_word_start {
            return false;
        }

        if self.cursor_position >= self.target_chars.len() {
            return false;
        }

        if self.should_complete() {
            self.complete();
            return false;
        }

        let is_correct = self.cursor_position < self.target_chars.len()
            && self.target_chars[self.cursor_position] == c;

        let current_char = self.target_chars[self.cursor_position];

        if !is_correct
            && self.cursor_position < self.target_chars.len()
            && self.target_chars[self.cursor_position] == ' '
        {
            self.register_keystroke(is_correct);
            return true;
        }

        self.register_keystroke(is_correct);

        // Memory management: Ensure we have capacity to avoid reallocations during fast typing
        if self.user_input.len() >= self.user_input.capacity() {
            let additional = std::cmp::max(100, self.user_input.capacity() / 2);
            self.user_input.reserve(additional);
            self.max_user_input_length = self.user_input.capacity();
        }

        self.user_input.push(Some(c));

        // lazyly track the correctnes of the current word
        if !is_correct && self.current_word_is_correct_so_far {
            self.current_word_is_correct_so_far = false;
            self.wrong_words_start_indexes
                .insert(self.current_word_start);
        }

        self.cursor_position += 1;

        // only validate correctnes of word at the boundaries
        if is_space && current_char == ' ' {
            if self
                .wrong_words_start_indexes
                .contains(&self.current_word_start)
            {
                self.validate_completed_word(self.current_word_start, self.cursor_position - 1);
            }

            self.current_word_start = self.cursor_position;
            self.current_word_is_correct_so_far = true;
        }

        if self.time_remaining.is_none() && self.cursor_position >= self.target_chars.len() {
            self.check_completion();
        }
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.status == Status::Completed {
            return false;
        }

        if self.cursor_position == 0 {
            return false;
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

    pub fn update_metrics(&mut self) {
        if self.status == Status::Completed || self.status == Status::Paused {
            return;
        }

        let Some(start_time) = self.time_started else {
            return;
        };

        if let Some(end_time) = self.time_end {
            self.time_remaining = Some(end_time.duration_since(Instant::now()));
        }

        self.accuracy = if self.total_keystrokes > 0 {
            ((self.correct_keystrokes as f64 / self.total_keystrokes as f64) * 100.0).round() as u8
        } else {
            0
        };

        let elapsed_seconds = start_time.elapsed().as_secs_f64();
        let elapsed_minutes = elapsed_seconds / 60.0;

        // no division by 0 on my watch - don't allow wpm calculation to reach levels over 9000
        if elapsed_seconds < 0.5 {
            return;
        }

        let chars_typed = self.user_input.len() as f64;
        let words_typed = chars_typed / 5.0;

        self.raw_wpm = words_typed / elapsed_minutes;

        let correct_words = (self.correct_keystrokes as f64) / 5.0;
        self.wpm = (correct_words / elapsed_minutes).max(0.0);

        let now = Instant::now();
        if now.duration_since(self.last_sample_time) >= Duration::from_secs(1) {
            self.wpm_samples.push(self.wpm.round() as u32);
            self.last_sample_time = now;
        }
    }

    /// Calculate typing consistency as a percentage.
    pub fn calculate_consistency(&self) -> f64 {
        if self.wpm_samples.len() < 2 {
            return 100.0;
        }

        let samples: Vec<f64> = self.wpm_samples.iter().map(|&x| x as f64).collect();
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;

        // shotout school
        let variance =
            samples.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (samples.len() - 1) as f64;
        let std_dev = variance.sqrt();

        (1.0 - (std_dev / mean)).clamp(0.0, 1.0) * 100.0
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
            self.completion_time = self.time_started.map(|start| start.elapsed().as_secs_f64());
            self.status = Status::Completed;
        }
        is_complete
    }

    fn register_keystroke(&mut self, is_correct: bool) {
        self.total_keystrokes += 1;
        if is_correct {
            self.correct_keystrokes += 1;
        }
    }

    pub fn complete(&mut self) {
        let start_time = self.time_started.unwrap_or(Instant::now());
        let end_time = self.time_end.unwrap_or(Instant::now());
        self.time_remaining = Some(Duration::from_secs(0));
        self.completion_time = Some(end_time.duration_since(start_time).as_secs_f64());
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
    fn validate_completed_word(&mut self, word_start: usize, word_end: usize) {
        let mut target_end = word_start;
        // TODO: need to improve the way we store things
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
    pub fn should_complete(&self) -> bool {
        if self.status != Status::Typing {
            return false;
        }
        if let Some(end_time) = self.time_end {
            Instant::now() >= end_time
        } else {
            false
        }
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

        assert_eq!(tracker.completion_time, Some(1.0));
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
            "Expected consistency around 90% for 10% variation, got {}%",
            consistency
        );

        // mean=50, std_dev=25 => variation would be 50% ==> consistentcy is 50%
        tracker.wpm_samples = vec![25, 50, 75];
        let consistency = tracker.calculate_consistency();
        assert!(
            (49.0..51.0).contains(&consistency),
            "Expected consistency around 50% for 50% variation, got {}%",
            consistency
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

        tracker.last_sample_time -= Duration::from_secs(1);
        tracker.time_started = Some(tracker.time_started.unwrap() - Duration::from_secs(1));
        tracker.update_metrics();
        assert_eq!(tracker.wpm_samples.len(), 1);

        for c in " world".chars() {
            tracker.type_char(c);
        }
        tracker.last_sample_time -= Duration::from_secs(1);
        tracker.time_started = Some(tracker.time_started.unwrap() - Duration::from_secs(1));
        tracker.update_metrics();
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
        tracker.last_sample_time = Instant::now() - Duration::from_secs(1);

        tracker.time_started = Some(tracker.time_started.unwrap() - Duration::from_secs(1));
        tracker.update_metrics();
        assert_eq!(
            tracker.wpm_samples.len(),
            1,
            "Should have collected first sample"
        );

        tracker.last_sample_time = Instant::now() - Duration::from_secs(2);

        tracker.time_started = Some(tracker.time_started.unwrap() - Duration::from_secs(1));
        tracker.update_metrics();
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
}
