use std::time::{Duration, Instant};

use crate::config::{Config, Mode};

#[derive(Debug)]
pub struct Tracker {
    // metrics
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: u8,

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
    pub wrong_words_start_indexes: std::collections::HashSet<usize>,
}

#[derive(Debug, PartialEq)]
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
        let target_chars: Vec<char> = target_text.chars().collect();
        let time_remaining = match mode {
            Mode::Time { duration } => Some(Duration::from_secs(duration)),
            Mode::Words { .. } => None,
        };

        Self {
            wpm: 0.0,
            raw_wpm: 0.0,
            accuracy: 0,
            time_remaining,
            time_started: None,
            time_paused: None,
            time_end: None,
            completion_time: None,
            user_input: Vec::new(),
            cursor_position: 0,
            target_text,
            target_chars,
            word_count,
            status: Status::Idle,
            total_keystrokes: 0,
            correct_keystrokes: 0,
            wrong_words_start_indexes: std::collections::HashSet::new(),
        }
    }

    pub fn start_typing(&mut self) {
        let now = Instant::now();
        self.time_started = Some(now);

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
        if self.cursor_position >= self.target_chars.len() {
            return false;
        }

        if self.should_complete() {
            self.complete();
            return false;
        }

        let is_correct = self.target_chars.get(self.cursor_position) == Some(&c);
        if !is_correct && self.target_chars.get(self.cursor_position) == Some(&' ') {
            self.register_keystroke(is_correct);
            return true;
        }

        self.register_keystroke(is_correct);
        self.user_input.push(Some(c));

        // we are about to type a wrong word
        if !is_correct {
            let current_word_start = self.get_current_word_start();
            if !self.wrong_words_start_indexes.contains(&current_word_start) {
                self.wrong_words_start_indexes.insert(current_word_start);
            }
        }

        self.cursor_position += 1;

        // only check completion if we have don't have a time limit
        if self.time_remaining.is_none() {
            self.check_completion();
        }
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor_position == 0 {
            return false;
        }

        // going back to a wrong word, unmark it
        let current_word_start = self.get_current_word_start();
        self.wrong_words_start_indexes.remove(&current_word_start);

        // allow backspace if we're at a mistyped character
        let current_input = self.user_input.get(self.cursor_position - 1);
        let target_char = self.target_chars.get(self.cursor_position - 1);

        if let (Some(Some(input_char)), Some(target_char)) = (current_input, target_char) {
            if *input_char != *target_char {
                self.user_input.pop();
                self.cursor_position -= 1;
                return true;
            }
        }

        // check if we're at a word boundary using character-based comparison
        let at_word_boundary = self.user_input.last() == Some(&Some(' '));

        // if we're not at a word boundary, or if the previous word was incorrect,
        // allow backspace
        if !at_word_boundary {
            self.user_input.pop();
            self.cursor_position -= 1;
            return true;
        }

        // check if the previous word was correct using character-based comparison
        let mut current_word_chars = Vec::new();
        let mut target_word_chars = Vec::new();
        let mut pos = current_word_start;

        // Collect characters for the current word from user input
        while pos < self.cursor_position - 1 {
            // -1 to exclude the space
            if let Some(Some(c)) = self.user_input.get(pos) {
                current_word_chars.push(*c);
            }
            pos += 1;
        }

        // Collect characters for the target word
        pos = current_word_start;
        while pos < self.target_chars.len() && self.target_chars[pos] != ' ' {
            target_word_chars.push(self.target_chars[pos]);
            pos += 1;
        }

        if current_word_chars != target_word_chars {
            self.user_input.pop();
            self.cursor_position -= 1;
            return true;
        }

        false
    }

    pub fn update_metrics(&mut self) {
        if self.status == Status::Completed || self.status == Status::Paused {
            return;
        }

        if self.should_complete() {
            self.complete();
            return;
        }

        let Some(start_time) = self.time_started else {
            return;
        };

        if let Some(end_time) = self.time_end {
            self.time_remaining = Some(end_time.duration_since(Instant::now()));
        }

        if !self.user_input.is_empty() {
            let elapsed_minutes = start_time.elapsed().as_secs_f64() / 60.0;

            self.accuracy = if self.total_keystrokes > 0 {
                ((self.correct_keystrokes as f64 / self.total_keystrokes as f64) * 100.0).round()
                    as u8
            } else {
                0
            };

            self.raw_wpm = (self.user_input.len() as f64 / 5.0) / elapsed_minutes;
            self.wpm = (self.correct_keystrokes as f64 / 5.0) / elapsed_minutes;
            self.wpm = self.wpm.max(0.0);
        }
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

    fn complete(&mut self) {
        let start_time = self.time_started.unwrap();
        let end_time = self.time_end.unwrap();
        self.time_remaining = Some(Duration::from_secs(0));
        self.completion_time = Some(end_time.duration_since(start_time).as_secs_f64());
        self.status = Status::Completed;
    }

    fn should_complete(&self) -> bool {
        if let Some(end_time) = self.time_end {
            Instant::now() >= end_time
        } else {
            false
        }
    }

    /// Returns the start index of the current word.
    fn get_current_word_start(&self) -> usize {
        let mut pos = self.cursor_position;
        while pos > 0 && self.target_chars.get(pos - 1) != Some(&' ') {
            pos -= 1;
        }
        pos
    }

    /// Returns true if the word at the given start index is marked as wrong.
    pub fn is_word_wrong(&self, start_idx: usize) -> bool {
        self.wrong_words_start_indexes.contains(&start_idx)
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
        assert_eq!(tracker.is_word_wrong(0), true);

        // bactrack
        for _ in 0.."hallo".len() {
            tracker.backspace();
        }
        assert_eq!(tracker.is_word_wrong(0), false);

        // Type "hello" correctly
        tracker.type_char('h');
        tracker.type_char('e');
        tracker.type_char('l');
        tracker.type_char('l');
        tracker.type_char('o');
        assert_eq!(tracker.is_word_wrong(0), false);
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

        assert_eq!(tracker.is_word_wrong(6), false);
        assert_eq!(tracker.is_word_wrong(12), true);

        // is first word still marked as wrong? should be
        assert_eq!(tracker.is_word_wrong(0), true);
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
        assert_eq!(tracker.correct_keystrokes, 5);
        assert_eq!(tracker.total_keystrokes, 7); // Including the wrong 'e' and backspace
    }
}
