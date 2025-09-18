use crate::{config::Mode, error::AppError, log_debug};
use std::time::{Duration, Instant};

/// Represents the current state of an individual typing test.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum TypingStatus {
    #[default]
    /// The typing test hasn't started. Idle state
    NotStarted,
    /// The typing test is currently in progress
    InProgress,
    /// The typing test is currently paused
    Paused,
    /// The typing test was un-paused and is awaiting input to resume typing test or bail out
    Resuming,
    /// The typing test has completed
    Completed,
}

/// Contains typing information about each token/chacter. A `Word` is composed of one or more `Tokens`
#[derive(Debug, Clone)]
pub struct Token {
    /// The typed token(char)
    pub typed: Option<char>,
    /// The actual expected token
    pub target: char,
    /// Wether this token was typed wrong or not
    pub is_wrong: bool,
    /// Time when this token was typed
    pub typed_at: Option<Instant>,
}

/// Contains information about a word present the the typing test word pool
#[derive(Debug, Clone)]
pub struct Word {
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
    /// The number of errors commited in the test
    pub total_errors: usize,
    /// The time the typing test started at
    pub start_time: Option<Instant>,
    /// The time the typing test ended at
    pub end_time: Option<Instant>,
    /// The time the typing test was paused at.
    paused_at: Option<Instant>,
    /// Total time the test has been paused
    total_paused_time: Duration,
    /// Metrics of the current test
    metrics: Metrics,
}

impl Tracker {
    pub fn new(text: String, mode: Mode) -> Self {
        let words = Self::build_words(&text);
        let tokens = Self::build_tokens(&text);
        if let Some(first_word) = words.first() {
            log_debug!("First word: {:?}", first_word);
        }
        if let Some(first_token) = tokens.first() {
            log_debug!("First token: {:?}", first_token);
        }

        Self {
            mode,
            status: TypingStatus::NotStarted,
            text,
            typed_text: String::new(),
            current_pos: 0,
            current_word_idx: 0,
            words,
            tokens,
            total_errors: 0,
            start_time: None,
            end_time: None,
            paused_at: None,
            total_paused_time: Duration::ZERO,
            metrics: Metrics::default(),
        }
    }

    pub fn reset(&mut self, text: String, mode: Mode) {
        *self = Self::new(text, mode);
    }

    fn build_words(text: &str) -> Vec<Word> {
        let text_vec: Vec<&str> = text.split_whitespace().collect();
        text_vec
            .iter()
            .map(|word| Word {
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
                target: chr,
                is_wrong: false,
                typed_at: None,
            })
            .collect()
    }

    pub fn start_typing(&mut self) {
        if matches!(
            self.status,
            TypingStatus::NotStarted | TypingStatus::Completed
        ) {
            let now = Instant::now();
            self.status = TypingStatus::InProgress;
            self.start_time = Some(now);
            if let Some(word) = self.words.get_mut(0) {
                word.start_time = Some(now);
            }
            self.invalidate_metrics_cache();
        }
    }

    fn resume(&mut self) {
        if let Some(paused_at) = self.paused_at {
            self.total_paused_time += Instant::now().duration_since(paused_at);
        }
        self.paused_at = None;
        self.status = TypingStatus::InProgress;
    }

    fn pause(&mut self) {
        // If already paused (e.g., from Resuming state), accumulate the previous pause time
        if let Some(paused_at) = self.paused_at {
            self.total_paused_time += Instant::now().duration_since(paused_at);
        }
        self.status = TypingStatus::Paused;
        self.paused_at = Some(Instant::now())
    }

    fn unpause(&mut self) {
        if self.start_time.is_some() {
            self.status = TypingStatus::Resuming;
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.status {
            TypingStatus::Paused => self.unpause(),
            TypingStatus::InProgress | TypingStatus::Resuming => self.pause(),
            _ => {}
        }
    }

    pub fn type_char(&mut self, c: char) -> Result<(), AppError> {
        let is_space = c == ' ';
        // never the first character of the test will be a space character.
        if self.is_idle() && is_space {
            return Err(AppError::IllegalSpaceCharacter);
        }

        // never the first character of a word is a space char, so if space is typed at the start
        // of a word do absolutely nothing. This is what monkeytype does anyways.
        if is_space && self.is_at_word_start() {
            return Err(AppError::IllegalSpaceCharacter);
        }

        // this the refractory state after unpausing the typing test, typing a char should continue
        // the test as per usual
        if self.is_resuming() {
            self.resume();
        }

        // if we get here we should auto-start typing as we are typing a valid chacter
        if self.is_idle() && !is_space {
            self.start_typing();
        }

        if !self.is_typing() {
            return Err(AppError::TypingTestNotInProgress);
        }

        if self.is_complete() {
            return Err(AppError::TypingTestAlreadyCompleted);
        }

        // this is the actual expected(target) character we are typing against
        let expected_char = self
            .tokens
            .get(self.current_pos)
            .ok_or(AppError::InvalidCharacterPosition)?
            .target;

        // upate current token information
        if let Some(token) = self.tokens.get_mut(self.current_pos) {
            token.typed = Some(c);
            token.typed_at = Some(Instant::now());
            token.is_wrong = expected_char != c;
        }

        self.typed_text.push(c);

        // errror tracking
        if expected_char != c {
            self.total_errors += 1;
            if let Some(word) = self.words.get_mut(self.current_word_idx) {
                word.error_count += 1;
            }
        }

        self.current_pos += 1;
        self.invalidate_metrics_cache();

        if self.should_mark_word_as_completed() {
            self.mark_word_as_completed();
        }

        if self.should_complete() {
            self.complete();
        }

        log_debug!("Tracker::type_char: {c}");
        Ok(())
    }

    pub fn backspace(&mut self) -> Result<(), AppError> {
        if !self.is_typing() {
            log_debug!("Tracker::backspace: TypingTestNotInProgress");
            return Err(AppError::TypingTestNotInProgress);
        }

        if self.current_pos == 0 {
            log_debug!("Tracker::backspace: IllegalBackspace");
            return Err(AppError::IllegalBackspace);
        }

        self.typed_text.pop();

        self.current_pos -= 1;

        // check if we are backspacing over a space that completed a word
        if let Some(token) = self.tokens.get(self.current_pos) {
            if token.target == ' ' && self.current_word_idx > 0 {
                // we are backspacing over a <space>, so go back to the previous word
                self.current_word_idx -= 1;
                if let Some(word) = self.words.get_mut(self.current_word_idx) {
                    word.completed = false;
                    word.end_time = None;
                }
            }
        }

        if let Some(token) = self.tokens.get_mut(self.current_pos) {
            token.typed = None;
            token.typed_at = None;
            token.is_wrong = false;

            if token.target != token.typed.unwrap_or('\0') {
                self.total_errors = self.total_errors.saturating_sub(1);
                if let Some(word) = self.words.get_mut(self.current_word_idx) {
                    word.error_count = word.error_count.saturating_sub(1);
                }
            }
        }
        self.invalidate_metrics_cache();
        log_debug!("Tracker::backspace: success");
        Ok(())
    }

    pub fn current_target_char(&self) -> Option<char> {
        self.tokens.get(self.current_pos).map(|c| c.target)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.status, TypingStatus::NotStarted)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.status, TypingStatus::Paused)
    }

    pub fn is_typing(&self) -> bool {
        matches!(self.status, TypingStatus::InProgress)
    }

    pub fn is_resuming(&self) -> bool {
        matches!(self.status, TypingStatus::Resuming)
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, TypingStatus::Completed)
    }

    pub fn check_completion(&mut self) {
        let typing_test_in_progress = self.is_typing() || self.is_resuming();
        if self.should_complete() && typing_test_in_progress {
            self.complete();
        }
    }

    fn should_mark_word_as_completed(&self) -> bool {
        if self.current_pos == 0 {
            return false;
        }
        let curr_char = self.tokens.get(self.current_pos - 1);
        let is_space_x = curr_char.map_or_else(|| false, |c| c.target == ' ');
        let is_end = self.current_pos >= self.text.len();

        is_space_x || is_end
    }

    // NOTE: i did this words end and start time because i think it would be nice to show in a
    // graph visualiation, but if this gets too annoying to deal with then remove it.
    fn mark_word_as_completed(&mut self) {
        if let Some(word) = self.words.get_mut(self.current_word_idx) {
            word.completed = true;
            word.end_time = Some(Instant::now());
        }
        self.current_word_idx += 1;

        if let Some(word) = self.words.get_mut(self.current_word_idx) {
            word.start_time = Some(Instant::now())
        }
    }

    pub fn should_complete(&self) -> bool {
        // all words are typed, should end test
        if self.current_pos >= self.text.len() {
            return true;
        }

        match self.mode {
            Mode::Time(secs) => {
                if self.start_time.is_some() {
                    self.elapsed_time() >= Duration::from_secs(secs.get() as u64)
                } else {
                    false
                }
            }
            Mode::Words(count) => self.current_word_idx >= count.get(),
        }
    }

    pub fn complete(&mut self) {
        self.end_time = Some(Instant::now());
        if let Some(word) = self.words.get_mut(self.current_word_idx) {
            if !word.completed {
                word.completed = true;
                word.end_time = Some(Instant::now())
            }
        }
        self.update_metrics();
        self.status = TypingStatus::Completed;
    }

    fn is_at_word_start(&self) -> bool {
        self.current_pos == 0
            || self.current_pos > 0
                && self
                    .tokens
                    .get(self.current_pos - 1)
                    .is_some_and(|prev| prev.target == ' ')
    }

    /// Returns an iterator over all words with their statistics
    pub fn words_iter(&self) -> impl Iterator<Item = &Word> {
        self.words.iter()
    }

    /// Returns the current WPM
    pub fn wpm(&mut self) -> f64 {
        self.try_metrics_update();
        self.metrics.wpm.unwrap_or(0.0)
    }

    /// Returns the current WPS (Words Per Second)
    pub fn wps(&mut self) -> f64 {
        self.wpm() / 60.0
    }

    /// Returns the current accuracy as a percentage (0.0 to 1.0)
    pub fn accuracy(&mut self) -> f64 {
        self.try_metrics_update();
        self.metrics.accuracy.unwrap_or(0.0)
    }

    pub fn consistency(&mut self) -> f64 {
        self.try_metrics_update();
        self.metrics.consistency.unwrap_or(0.0)
    }

    /// Returns a summary of the current typing session
    pub fn summary(&mut self) -> Summary {
        self.try_metrics_update();

        Summary {
            wpm: self.wpm(),
            wps: self.wps(),
            accuracy: self.accuracy(),
            consistency: self.consistency(),
            total_chars: self.text.len(),
            correct_chars: self.correct_chars_count(),
            total_errors: self.total_errors,
            elapsed_time: self.elapsed_time(),
            total_paused_time: self.total_paused_time,
            completed_words: self.words.iter().filter(|w| w.completed).count(),
            total_words: self.words.len(),
            progress: self.progress(),
            is_completed: self.is_complete(),
        }
    }

    /// Returns the current test progress. Takes into consideration the test mode for the progress calculation
    pub fn progress(&self) -> f64 {
        match self.mode {
            Mode::Words(_) => {
                if self.text.is_empty() {
                    return 1.0;
                }
                self.current_pos as f64 / self.text.len() as f64
            }
            Mode::Time(total_seconds) => {
                if self.status == TypingStatus::Completed {
                    1.0
                } else if let Some(start) = self.start_time {
                    let elapsed = start.elapsed().as_secs_f64();
                    (elapsed / total_seconds.get() as f64).min(1.0)
                } else {
                    0.0
                }
            }
        }
    }

    /// Returns the elapsed time of the curren typin test
    pub fn elapsed_time(&self) -> Duration {
        let raw_elapsed = match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => end.duration_since(start),
            (Some(start), None) => start.elapsed(),
            _ => Duration::ZERO,
        };
        let mut adjusted = raw_elapsed.saturating_sub(self.total_paused_time);
        if let Some(paused_at) = self.paused_at {
            adjusted = adjusted.saturating_sub(paused_at.elapsed());
        }
        adjusted
    }

    pub fn correct_chars_count(&self) -> usize {
        self.typed_text.len() - self.total_errors
    }

    fn try_metrics_update(&mut self) {
        if self.is_complete() || self.is_paused() || self.is_resuming() {
            return;
        }

        let now = Instant::now();
        let should_update = self.metrics.last_updated_at.map_or_else(
            || true,
            |last| {
                // TODO: check if we should increase this
                now.duration_since(last) > Duration::from_millis(100)
            },
        );

        // let should_update = self.metrics.last_updated_at.map_or(true, |last| {
        //     // TODO: check if we should increase this
        //     now.duration_since(last) > Duration::from_millis(100)
        // });
        if should_update {
            self.update_metrics();
            self.metrics.last_updated_at = Some(now)
        }
    }

    fn update_metrics(&mut self) {
        self.metrics.wpm = Some(self.calculate_wpm());
        self.metrics.accuracy = Some(self.calculate_accuracy());
        self.metrics.consistency = Some(self.calcluate_consistency());
    }

    fn calculate_wpm(&self) -> f64 {
        let total_chars_typed = self.typed_text.len() as f64;
        if total_chars_typed <= 0.0 {
            return 0.0;
        }

        let elapsed_secs = self.elapsed_time().as_secs_f64();
        let effective_secs = elapsed_secs.max(0.1); // avoid spikes on short time/word tests

        // wpm = (chars / 5) per minute
        (total_chars_typed / 5.0) / (effective_secs / 60.0)
    }

    fn calculate_accuracy(&self) -> f64 {
        let total_typed = self.typed_text.len() as f64;
        if total_typed > 0.0 {
            self.correct_chars_count() as f64 / total_typed
        } else {
            0.0
        }
    }

    fn calcluate_consistency(&self) -> f64 {
        let completed_words: Vec<_> = self
            .words
            .iter()
            .filter(|w| w.completed && w.start_time.is_some() && w.end_time.is_some())
            .collect();

        if completed_words.len() < 2 {
            return 0.0; // perfect consistency with 0 or 1 words
        }

        let word_speeds: Vec<f64> = completed_words
            .iter()
            .filter_map(|word| {
                let duration = word
                    .end_time?
                    .duration_since(word.start_time?)
                    .as_secs_f64();
                let chars = word.target.len() as f64;
                if duration > 0.0 {
                    Some((chars / 5.0) / (duration / 60.0)) // WPM for this word
                } else {
                    None
                }
            })
            .collect();

        if word_speeds.is_empty() {
            return 0.0;
        }

        // std dev
        let mean = word_speeds.iter().sum::<f64>() / word_speeds.len() as f64;
        let variance = word_speeds
            .iter()
            .map(|speed| (speed - mean).powi(2))
            .sum::<f64>()
            / word_speeds.len() as f64;

        variance.sqrt()
    }

    fn invalidate_metrics_cache(&mut self) {
        self.metrics = Metrics::default();
    }
}

// TODO: add more metrics such as raw_wpm and such
#[derive(Debug, Default, Clone)]
struct Metrics {
    wpm: Option<f64>,
    accuracy: Option<f64>,
    consistency: Option<f64>,
    last_updated_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct Summary {
    pub wpm: f64,
    pub wps: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub total_chars: usize,
    pub total_words: usize,
    pub total_errors: usize,
    pub correct_chars: usize,
    pub elapsed_time: Duration,
    pub total_paused_time: Duration,
    pub completed_words: usize,
    pub progress: f64,
    pub is_completed: bool,
}

impl Summary {
    pub fn net_wpm(&self) -> f64 {
        if self.accuracy > 0.0 {
            self.wpm * self.accuracy
        } else {
            0.0
        }
    }

    pub fn error_percentage(&self) -> f64 {
        if self.total_chars > 0 {
            (self.total_errors as f64 / self.total_chars as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn completion_percentage(&self) -> f64 {
        self.progress * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;

    #[test]
    fn test_new_tracker() {
        let text = "hello termitype".to_string();
        let mode = Mode::with_time(60);
        let tracker = Tracker::new(text.clone(), mode);

        assert_eq!(tracker.text, text);
        assert_eq!(tracker.mode, mode);
        assert_eq!(tracker.current_pos, 0);
        assert_eq!(tracker.total_errors, 0);
        assert_eq!(tracker.status, TypingStatus::NotStarted);
    }

    #[test]
    fn test_start_typing() {
        let mut tracker = Tracker::new("termitype".to_string(), Mode::with_time(5));
        tracker.start_typing();
        assert_eq!(tracker.status, TypingStatus::InProgress);
        assert!(tracker.start_time.is_some());
    }

    #[test]
    fn test_status_change() {
        let mut tracker = Tracker::new("random".to_string(), Mode::with_time(60));
        assert_eq!(tracker.status, TypingStatus::NotStarted);
        tracker.toggle_pause();
        assert_eq!(tracker.status, TypingStatus::NotStarted);
        tracker.toggle_pause();

        tracker.start_typing();
        tracker.toggle_pause();
        assert_eq!(tracker.status, TypingStatus::Paused);
        tracker.toggle_pause();
        assert_eq!(tracker.status, TypingStatus::Resuming);
        tracker.toggle_pause();
        assert!(tracker.type_char('c').is_err());
        assert_eq!(tracker.status, TypingStatus::Paused);
        tracker.toggle_pause();
        assert_eq!(tracker.status, TypingStatus::Resuming);
        tracker.type_char('c').unwrap();
        assert_eq!(tracker.status, TypingStatus::InProgress);
    }

    #[test]
    fn test_type_correct_char() {
        let mut tracker = Tracker::new("hi".to_string(), Mode::with_time(5));
        tracker.start_typing();

        assert!(tracker.type_char('h').is_ok());
        assert_eq!(tracker.current_pos, 1);
        assert_eq!(tracker.typed_text, "h");
        assert_eq!(tracker.total_errors, 0);
    }

    #[test]
    fn test_type_incorrect_char() {
        let mut tracker = Tracker::new("hi".to_string(), Mode::with_time(5));
        tracker.start_typing();
        assert!(tracker.type_char('e').is_ok());
        assert_eq!(tracker.current_pos, 1);
        assert_eq!(tracker.typed_text, "e");
        assert_eq!(tracker.total_errors, 1);
    }

    #[test]
    fn test_complete_short_word_typing_test_with_time_mode() {
        let mut tracker = Tracker::new("hi".to_string(), Mode::with_time(60));
        tracker.start_typing();

        tracker.type_char('h').unwrap();
        tracker.type_char('i').unwrap();

        assert!(tracker.is_complete());
        assert_eq!(tracker.status, TypingStatus::Completed);
    }

    #[test]
    fn test_word_mode_completion() {
        let mut tracker = Tracker::new("hello world".to_string(), Mode::with_words(2));
        tracker.start_typing();
        for c in "hello ".chars() {
            tracker.type_char(c).unwrap()
        }

        assert!(!tracker.is_complete());

        for c in "world".chars() {
            tracker.type_char(c).unwrap()
        }
        assert!(tracker.is_complete())
    }

    #[test]
    fn test_progress_words_mode() {
        let text = "testing termitype";
        let mut tracker = Tracker::new(
            text.to_string(),
            Mode::Words(NonZeroUsize::new(10).unwrap()),
        );
        tracker.current_pos = 5;
        assert_eq!(tracker.progress(), 5.0 / text.len() as f64);

        tracker.current_pos = text.len();
        assert_eq!(tracker.progress(), 1.0);

        let empty_tracker =
            Tracker::new("".to_string(), Mode::Words(NonZeroUsize::new(10).unwrap()));
        assert_eq!(empty_tracker.progress(), 1.0);
    }

    #[test]
    fn test_progress_time_mode() {
        let total_seconds = 10;
        let mut tracker = Tracker::new(
            "test".to_string(),
            Mode::Time(NonZeroUsize::new(total_seconds).unwrap()),
        );

        // NotStarted
        assert_eq!(tracker.progress(), 0.0);

        // InProgress: 5 seconds in
        tracker.start_time = Some(Instant::now() - Duration::from_secs(5));
        tracker.status = TypingStatus::InProgress;
        let progress = tracker.progress();
        assert!((0.4..=0.6).contains(&progress)); // approximate due to timing

        // Completed
        tracker.status = TypingStatus::Completed;
        assert_eq!(tracker.progress(), 1.0);

        // elapsed more than total test time, this shouldnt not happen but just in case we want
        // it to cap the progress at 100%
        tracker.start_time = Some(Instant::now() - Duration::from_secs(15));
        tracker.status = TypingStatus::InProgress;
        assert_eq!(tracker.progress(), 1.0);
    }

    #[test]
    fn test_summary() {
        let str = "hello termitype".to_string();
        let mut tracker = Tracker::new(str.clone(), Mode::with_time(60));
        tracker.start_typing();

        tracker.type_char('h').unwrap();
        tracker.type_char('e').unwrap();
        tracker.type_char('l').unwrap();
        tracker.type_char('x').unwrap(); // error
        tracker.type_char('o').unwrap();

        let summary = tracker.summary();
        assert!(summary.wpm >= 0.0);
        assert!(summary.wps >= 0.0);
        assert!(summary.accuracy > 0.0);
        assert_eq!(summary.total_chars, str.len());
        assert_eq!(summary.correct_chars, 4);
        assert_eq!(summary.total_errors, 1);
        assert!(summary.elapsed_time > Duration::ZERO);
        // wps shoud be wpm over 60
        assert!((summary.wps - summary.wpm / 60.0).abs() < 0.001);
    }

    #[test]
    fn test_reset() {
        let mut tracker = Tracker::new("hello world".to_string(), Mode::with_time(60));
        tracker.start_typing();
        tracker.type_char('h').unwrap();
        tracker.type_char('e').unwrap();

        assert_eq!(tracker.status, TypingStatus::InProgress);
        assert_eq!(tracker.current_pos, 2);
        assert_eq!(tracker.typed_text, "he");
        assert_eq!(tracker.total_errors, 0);
        assert!(tracker.start_time.is_some());

        let new_text = "new test".to_string();
        let new_mode = Mode::with_words(5);
        tracker.reset(new_text.clone(), new_mode);

        assert_eq!(tracker.status, TypingStatus::NotStarted);
        assert_eq!(tracker.text, new_text);
        assert_eq!(tracker.mode, new_mode);
        assert_eq!(tracker.current_pos, 0);
        assert_eq!(tracker.current_word_idx, 0);
        assert_eq!(tracker.typed_text, "");
        assert_eq!(tracker.total_errors, 0);
        assert!(tracker.start_time.is_none());
        assert!(tracker.end_time.is_none());
        assert_eq!(tracker.words.len(), 2); // "new test"
        assert_eq!(tracker.tokens.len(), new_text.len());
    }

    #[test]
    fn test_word_index() {
        let mut tracker = Tracker::new("hi you test".to_string(), Mode::with_words(3));
        tracker.start_typing();
        tracker.type_char('h').unwrap();
        tracker.type_char('i').unwrap();
        tracker.type_char(' ').unwrap();
        tracker.backspace().unwrap();
        tracker.type_char(' ').unwrap();
        tracker.backspace().unwrap();
        assert_eq!(tracker.current_word_idx, 0);
    }

    #[test]
    fn test_token_tracking() {
        let mut tracker = Tracker::new("hi you test".to_string(), Mode::with_words(3));
        tracker.start_typing();

        assert!(!tracker.tokens.first().unwrap().is_wrong);
        assert!(tracker.tokens.first().unwrap().typed.is_none());
        assert_eq!(tracker.tokens.first().unwrap().target, 'h');

        tracker.type_char('f').unwrap();
        assert!(tracker.tokens.first().unwrap().is_wrong);
        assert!(tracker.tokens.first().unwrap().typed.is_some());
        assert_eq!(tracker.tokens.first().unwrap().typed, Some('f'));
        assert_eq!(tracker.tokens.first().unwrap().target, 'h');

        tracker.backspace().unwrap();
        assert!(!tracker.tokens.first().unwrap().is_wrong);
        assert!(tracker.tokens.first().unwrap().typed.is_none());
        assert_eq!(tracker.tokens.first().unwrap().target, 'h');

        tracker.type_char('h').unwrap();
        assert!(!tracker.tokens.first().unwrap().is_wrong);
        assert!(tracker.tokens.first().unwrap().typed.is_some());
        assert_eq!(tracker.tokens.first().unwrap().typed, Some('h'));
    }

    #[test]
    fn test_word_tracking() {
        let mut tracker = Tracker::new("hi you test".to_string(), Mode::with_words(3));
        tracker.start_typing();

        assert!(!tracker.words.first().unwrap().completed);
        assert_eq!(tracker.words.first().unwrap().error_count, 0);

        tracker.type_char('n').unwrap();
        assert!(!tracker.words.first().unwrap().completed);
        assert_eq!(tracker.words.first().unwrap().error_count, 1);

        tracker.type_char('o').unwrap();
        assert!(!tracker.words.first().unwrap().completed);
        assert_eq!(tracker.words.first().unwrap().error_count, 2);

        tracker.backspace().unwrap();
        assert_eq!(tracker.words.first().unwrap().error_count, 1);
        tracker.backspace().unwrap();
        assert_eq!(tracker.words.first().unwrap().error_count, 0);

        tracker.type_char('h').unwrap();
        tracker.type_char('i').unwrap();
        tracker.type_char(' ').unwrap(); // we only mark a word as completed after we move from it with <space>
        assert_eq!(tracker.words.first().unwrap().error_count, 0);
        assert!(tracker.words.first().unwrap().completed);
    }

    #[test]
    fn test_illegal_space() {
        let mut tracker = Tracker::new("hello world".to_string(), Mode::with_time(60));
        let space_input = tracker.type_char(' ');
        println!("space_input: {:?}", space_input);
        assert!(matches!(space_input, Err(AppError::IllegalSpaceCharacter))); // error
        assert_eq!(tracker.status, TypingStatus::NotStarted);

        tracker.type_char('h').unwrap();
        tracker.type_char('e').unwrap();
        tracker.type_char('l').unwrap();
        tracker.type_char('l').unwrap();
        tracker.type_char('o').unwrap();
        let space_input = tracker.type_char(' '); // we have typed `hello `
        assert!(!matches!(space_input, Err(AppError::IllegalSpaceCharacter))); // not an error
        assert_eq!(tracker.current_pos, 6);

        let space_input = tracker.type_char(' ');
        assert!(matches!(space_input, Err(AppError::IllegalSpaceCharacter))); // error
        assert_eq!(tracker.current_pos, 6);
        assert_eq!(tracker.typed_text, "hello ");
    }

    #[test]
    fn test_pause_time_tracking() {
        let mut tracker = Tracker::new("test".to_string(), Mode::with_time(60));
        tracker.type_char('t').unwrap();
        assert_eq!(tracker.status, TypingStatus::InProgress);
        assert_eq!(tracker.total_paused_time, Duration::ZERO);

        // pause the typing test
        tracker.toggle_pause();
        assert_eq!(tracker.status, TypingStatus::Paused);
        assert!(tracker.paused_at.is_some());

        // simulate a 100ms pause
        tracker.paused_at = Some(Instant::now() - Duration::from_millis(100));

        // unpause
        tracker.toggle_pause();
        assert_eq!(tracker.status, TypingStatus::Resuming);
        assert!(tracker.paused_at.is_some());
        assert_eq!(tracker.total_paused_time, Duration::ZERO); // we don't update paused_tiem until we get to `InProgress`

        // we resume the typing test
        tracker.type_char('e').unwrap();
        assert_eq!(tracker.status, TypingStatus::InProgress);
        assert!(tracker.paused_at.is_none());
        assert!(tracker.total_paused_time >= Duration::from_millis(100)); // now that we are in progress again we add to the paused time total

        // pause again, simulate 500ms pause
        tracker.toggle_pause();
        tracker.paused_at = Some(Instant::now() - Duration::from_millis(500));
        tracker.toggle_pause();
        tracker.type_char('s').unwrap();

        assert!(tracker.total_paused_time >= Duration::from_millis(600));

        let summary = tracker.summary();
        assert!(summary.elapsed_time < Duration::from_millis(50));
        assert!(summary.total_paused_time >= Duration::from_millis(600));
        assert!(summary.total_paused_time <= Duration::from_millis(601));
    }

    #[test]
    fn test_word_mode_fast_wpm_calculation() {
        let mut tracker = Tracker::new("test".to_string(), Mode::with_words(1));

        for c in "test".chars() {
            tracker.type_char(c).unwrap();
        }

        let summary = tracker.summary();
        assert_eq!(tracker.status, TypingStatus::Completed);
        assert!(summary.wpm > 0.0);

        // 4 chars = 0.8 words, in 0.1 seconds = 0.8 / (0.1/60) = 480 WPM
        assert_eq!(summary.wpm, 480.0);
    }
}
