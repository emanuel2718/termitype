/*
    state.rs  -> tracker.rs
    TestState -> TypingStatus
    CharInfo   -> Token
    CharInfo.has_error => Token.is_wrong
    WordStats  -> Word
    TypingState -> Tracker
*/

use crate::{config::Mode, log_debug};
use std::time::Instant;

/// Represents the current state of an individual typing test.
#[derive(Debug, Default, Clone)]
pub enum TypingStatus {
    #[default]
    /// The typing test hasn't started. Idle state
    NotStarted,
    /// The typing test is currently in progress
    InProgress,
    /// The typing test is currently paused
    Paused,
    /// The typing test has completed
    Completed,
}

/// Contains typing information about each token/chacter. A `Word` is composed of one or more `Tokens`
#[derive(Debug, Clone)]
pub struct Token {
    /// The typed token(char)
    pub typed: Option<char>,
    /// The actual expected token
    pub expected: char,
    /// Wether this token was typed wrong or not
    pub is_wrong: bool,
    /// Time when this token was typed
    pub typed_at: Option<Instant>,
}

/// Contains information about a word present the the typing test word pool
#[derive(Debug, Clone)]
pub struct Word {
    /// The current typed text for the word
    pub typed: String,
    /// The target word text
    pub target: String,
    /// Time when this word was started
    pub start_time: Option<Instant>,
    /// Time when this word was completed
    pub end_time: Option<Instant>,
    /// Number of errors found in the word
    pub error_count: usize,
    /// Wether this word has been typed completely or not
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct Tracker {
    /// The mode of the current typing test
    pub mode: Mode,
    /// Current test status
    pub status: TypingStatus,
    /// The acutal target text to type against
    pub text: String,
    /// The current text typed in the current test
    pub typed_text: String,
    /// The current token position in the text
    pub current_pos: usize,
    /// The current word position
    pub current_word_idx: usize,
    /// Information about each of the words on the test
    pub words: Vec<Word>,
    /// Information about each fo the tokens(chars) on the test
    pub tokens: Vec<Token>,
    /// The time the typing test started at
    pub start_time: Option<Instant>,
    /// The time the typing test ended at
    pub end_time: Option<Instant>,
    /// The number of errors commited in the test
    pub total_errors: usize,
    /// Metrics of the current test
    metrics: Metrics,
}

// TODO: add more metrics such as raw_wpm and such
#[derive(Debug, Default, Clone)]
struct Metrics {
    wpm: Option<f64>,
    accuracy: Option<f64>,
    consistency: Option<f64>,
    last_updated_at: Option<Instant>,
}

impl Tracker {
    pub fn new(text: String, mode: Mode) -> Self {
        let words = Self::build_words(&text);
        let tokens = Self::build_tokens(&text);
        log_debug!("First word: {:?}", words[0]);
        log_debug!("First token: {:?}", tokens[0]);

        Self {
            mode,
            status: TypingStatus::NotStarted,
            text,
            typed_text: String::new(),
            current_pos: 0,
            current_word_idx: 0,
            words,
            tokens,
            start_time: None,
            end_time: None,
            total_errors: 0,
            metrics: Metrics::default(),
        }
    }

    fn build_words(text: &str) -> Vec<Word> {
        let text_vec: Vec<&str> = text.split_whitespace().collect();
        text_vec
            .iter()
            .map(|word| Word {
                typed: String::new(),
                target: word.to_string(),
                start_time: None,
                end_time: None,
                error_count: 0,
                completed: false,
            })
            .collect()
    }

    fn build_tokens(text: &str) -> Vec<Token> {
        text.chars()
            .map(|chr| Token {
                typed: None,
                expected: chr,
                is_wrong: false,
                typed_at: None,
            })
            .collect()
    }
}
