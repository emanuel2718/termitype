use std::time::{Duration, Instant};

use crate::config::{Config, Mode};

#[derive(Debug)]
pub struct Tracker {
    // metrics
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: u8,
    total_keystrokes: usize,
    correct_keystrokes: usize,

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
    pub word_count: usize,
    pub status: Status,
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Idle,
    Typing,
    Paused,
    Completed,
}

/*

wrong words:  could be represented as a queue of indexes. Where the index is the index of the word in the target text.

if we register a incorrectly typed char,
for each word, in between <space>






*/

impl Tracker {
    pub fn new(config: &Config, target_text: String) -> Self {
        let mode = config.current_mode();
        let word_count = mode.value();
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
            word_count,
            status: Status::Idle,
            total_keystrokes: 0,
            correct_keystrokes: 0,
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
        if self.cursor_position >= self.target_text.len() {
            return false;
        }

        let is_correct = self.target_text.chars().nth(self.cursor_position) == Some(c);
        if !is_correct && self.target_text.chars().nth(self.cursor_position) == Some(' ') {
            self.register_keystroke(is_correct);
            return true;
        }

        self.register_keystroke(is_correct);
        self.user_input.push(Some(c));
        self.cursor_position += 1;
        self.check_completion();
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor_position == 0 {
            return false;
        }
        // allow backspace if we're at a mistyped character
        let current_input = self.user_input.get(self.cursor_position - 1);
        let target_char = self.target_text.chars().nth(self.cursor_position - 1);

        if let (Some(Some(input_char)), Some(target_char)) = (current_input, target_char) {
            if *input_char != target_char {
                self.user_input.pop();
                self.cursor_position -= 1;
                return true;
            }
        }

        // check if we're at a word boundary
        let input: String = self.user_input.iter().filter_map(|&x| x).collect();
        let at_word_boundary = input.ends_with(' ');

        // if we're not at a word boundary, or if the previous word was incorrect,
        // allow backspace
        if !at_word_boundary {
            self.user_input.pop();
            self.cursor_position -= 1;
            return true;
        }

        // check if the previous word was correct
        let target_words: Vec<&str> = self.target_text.split_whitespace().collect();
        let input_words: Vec<&str> = input.split_whitespace().collect();
        let current_word_idx = input_words.len().saturating_sub(1);

        if input_words.get(current_word_idx) != target_words.get(current_word_idx) {
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
            let now = Instant::now();
            if now >= end_time {
                self.time_remaining = Some(Duration::from_secs(0));
                self.completion_time = Some(end_time.duration_since(start_time).as_secs_f64());
                self.status = Status::Completed;
                return;
            }
            self.time_remaining = Some(end_time.duration_since(now));
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
            None => self.cursor_position >= self.target_text.len(),
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
}
