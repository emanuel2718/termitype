use std::time::{Duration, Instant};

use crate::config::{Config, Mode};

#[derive(Debug)]
pub struct Tracker {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuraccy: u8,
    pub consistency: u8,
    // test_type: Something
    pub time_started: Option<Instant>,
    pub time_remaining: Option<Duration>,
    pub word_count: usize,
    pub user_input: Vec<Option<char>>,
    pub target_text: String,
    pub cursor_position: usize,
    pub status: Status,
    pub wrong_words: Vec<String>,
    pub wrong_chars: Vec<char>,
}

#[derive(Debug)]
pub enum Status {
    Idle,
    Typing,
    Paused,
    Completed,
}

impl Tracker {
    pub fn new(config: &Config) -> Self {
        let mode = config.resolve_mode();
        let word_count = match mode {
            Mode::Time { .. } => 100,
            Mode::Words { count } => count,
        };
        let (target_text, _) = match mode {
            Mode::Time { duration } => ("hello time".to_string(), duration),
            Mode::Words { .. } => ("hello words".to_string(), 0),
        };
        Tracker {
            wpm: 0.0,
            raw_wpm: 0.0,
            accuraccy: 0,
            consistency: 0,
            time_started: None,
            time_remaining: None,
            word_count,
            user_input: Vec::new(),
            target_text,
            cursor_position: 0,
            status: Status::Idle,
            wrong_words: Vec::new(),
            wrong_chars: Vec::new(),
        }
    }
}
