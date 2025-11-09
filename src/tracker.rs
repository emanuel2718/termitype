use crate::{
    config::Mode, constants::MAX_EXTRA_WRONG_CHARS, error::AppError, log_debug, notifications,
};
use std::time::{Duration, Instant};

const WORD_BOUNDARY_ESTIMATE_RATIO: usize = 5;
const DEFAULT_WORD_BOUNDARY_CAPACITY: usize = 16;

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
    /// Whether this token was skipped during space jump or not
    pub is_skipped: bool,
    /// Time when this token was typed
    pub typed_at: Option<Instant>,
}

impl Token {
    pub fn is_extra_token(&self) -> bool {
        self.is_wrong && self.target == self.typed.unwrap_or('\0') && self.target != ' '
    }

    fn is_correct_non_space_token(&self) -> bool {
        self.target != ' ' && self.typed == Some(self.target) && !self.is_wrong
    }
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

/// Represents a space jump operation
#[derive(Debug, Clone, Copy, PartialEq)]
struct SpaceJump {
    /// Position before the jump
    source_pos: usize,
    /// Position after the jump
    target_pos: usize,
}

impl SpaceJump {
    #[inline]
    const fn new(source_pos: usize, target_pos: usize) -> Self {
        Self {
            source_pos,
            target_pos,
        }
    }
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
    /// Tracks positions where space jumps occurred
    space_jump_stack: Vec<SpaceJump>,
    /// Pre-calculated indexed of word boundaries
    word_boundaries: Vec<usize>,
    /// Number of extra wrong tokens added at the word boundaries
    extra_errors_count: usize,
    /// Snapshots of WPM samples taken every second while typing
    wpm_snapshots: WpmSnapshots,
    /// Last time a WPM sample snapshot was taken
    last_snapshot_time: Option<Instant>,
}

impl Tracker {
    pub fn new(text: String, mode: Mode) -> Self {
        let words = Self::build_words(&text);
        let tokens = Self::build_tokens(&text);
        let word_boundaries = Self::build_word_boundaries(&text);

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
            space_jump_stack: Vec::new(),
            word_boundaries,
            metrics: Metrics::default(),
            wpm_snapshots: WpmSnapshots::new(),
            extra_errors_count: 0,
            last_snapshot_time: None,
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
                is_skipped: false,
                typed_at: None,
            })
            .collect()
    }

    /// Bulids the word boundaries vec, basically all the position where each word starts after spce
    fn build_word_boundaries(text: &str) -> Vec<usize> {
        let capacity =
            (text.len() / WORD_BOUNDARY_ESTIMATE_RATIO).max(DEFAULT_WORD_BOUNDARY_CAPACITY);
        let mut boundaries = Vec::with_capacity(capacity);

        boundaries.push(0); // initial pos

        let mut search_pos = 0;
        while let Some(space_pos) = text[search_pos..].find(' ') {
            let word_start = search_pos + space_pos + 1;
            if word_start < text.len() {
                boundaries.push(word_start);
                search_pos = word_start;
            } else {
                break;
            }
        }

        boundaries.shrink_to_fit();
        boundaries
    }

    pub fn start_typing(&mut self) {
        if matches!(
            self.status,
            TypingStatus::NotStarted | TypingStatus::Completed
        ) {
            let now = Instant::now();
            self.status = TypingStatus::InProgress;
            self.start_time = Some(now);
            if let Some(word) = self.current_word_mut() {
                word.start_time = Some(now);
            }
            self.invalidate_metrics_cache();
            self.update_metrics();
            // NOTE: maybe we want a configurable option to let the user determine this behavior
            // for now, just clear them on test start
            // Maybe this is a weird side-effect, not sure if I like this
            if notifications::has_any() {
                notifications::clear_notifications();
            }
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

        // resume the test if paused or in refractory state after unpausing
        if self.is_resuming() || self.is_paused() {
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
            .current_token()
            .ok_or(AppError::InvalidCharacterPosition)?
            .target;

        // add wrong tokens at word boundary. Monkey see, monkey do...
        if expected_char == ' ' && !is_space {
            if self.extra_errors_count < MAX_EXTRA_WRONG_CHARS {
                let new_token = Token {
                    typed: Some(c),
                    target: c,
                    is_wrong: true,
                    is_skipped: false,
                    typed_at: Some(Instant::now()),
                };
                self.tokens.insert(self.current_pos, new_token);
                self.typed_text.push(c);
                self.total_errors += 1;
                // Note: we don't increment word.error_count for extra tokens at word boundaries
                // because they are boundary errors, not errors within the word itself
                self.extra_errors_count += 1;
                self.current_pos += 1;
            }
            return Ok(());
        }

        // space jumping shenanigans
        if is_space && expected_char != ' ' {
            if let Some(target_pos) = self.calculate_space_jump_target() {
                return self.perform_space_jump(target_pos);
            }
        }

        // upate current token information
        if let Some(token) = self.current_token_mut() {
            token.typed = Some(c);
            token.typed_at = Some(Instant::now());
            token.is_wrong = expected_char != c;
        }

        self.typed_text.push(c);

        // errror tracking
        if expected_char != c {
            self.total_errors += 1;
            if let Some(word) = self.current_word_mut() {
                word.error_count += 1;
            }
        }

        self.current_pos += 1;

        if self.should_mark_word_as_completed() {
            self.mark_word_as_completed();
        }

        // if self.should_complete() {
        //     self.complete();
        // }

        log_debug!("Tracker::type_char: {c}");
        Ok(())
    }

    pub fn backspace(&mut self) -> Result<(), AppError> {
        // resume the test if paused or in refractory state after unpausing
        if self.is_resuming() || self.is_paused() {
            self.resume();
        }

        if !self.is_typing() {
            log_debug!("Tracker::backspace: TypingTestNotInProgress");
            return Err(AppError::TypingTestNotInProgress);
        }

        if self.current_pos == 0 {
            log_debug!("Tracker::backspace: IllegalBackspace");
            return Err(AppError::IllegalBackspace);
        }

        // disallow backspace at word boundary after a correctly typed word,
        // but allow backspacing over space-jumped words or words with extra tokens
        if self.is_previous_token_a_space() {
            if let Some(prev_word) = self.prev_word() {
                if prev_word.completed
                    && prev_word.error_count == 0
                    && !self.prev_word_has_skipped_tokens()
                    && !self.prev_word_has_extra_tokens()
                {
                    return Ok(());
                }
            }
        }

        // undo space jump action
        if let Some(&jump) = self.space_jump_stack.last() {
            if self.current_pos == jump.target_pos {
                return self.undo_space_jump(jump);
            }
        }

        self.typed_text.pop();

        self.current_pos -= 1;

        // are we currently backspacing over an *extra* wrong token question mark
        if let Some(token) = self.tokens.get(self.current_pos) {
            if token.is_extra_token() {
                self.tokens.remove(self.current_pos);
                self.total_errors = self.total_errors.saturating_sub(1);
                self.extra_errors_count = self.extra_errors_count.saturating_sub(1);
                return Ok(()); // do not process the extra token
            }
        }

        // if we are backspacing over a space that completed a word, unmark the word as completed
        if let Some(token) = self.current_token() {
            if token.target == ' ' && token.typed.is_some() && self.current_word_idx > 0 {
                self.current_word_idx -= 1;
                if let Some(word) = self.current_word_mut() {
                    word.completed = false;
                    word.end_time = None;
                }
            }
        }

        if let Some(token) = self.current_token_mut() {
            let was_wrong = token.target != token.typed.unwrap_or('\0');
            token.typed = None;
            token.typed_at = None;
            token.is_wrong = false;
            token.is_skipped = false;

            if was_wrong {
                self.total_errors = self.total_errors.saturating_sub(1);
                if let Some(word) = self.current_word_mut() {
                    word.error_count = word.error_count.saturating_sub(1);
                }
            }
        }
        log_debug!("Tracker::backspace: success");
        Ok(())
    }

    pub fn current_target_char(&self) -> Option<char> {
        self.current_token().map(|c| c.target)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.status, TypingStatus::NotStarted)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.status, TypingStatus::Paused)
    }

    pub fn in_progress(&self) -> bool {
        matches!(self.status, TypingStatus::InProgress)
            || matches!(self.status, TypingStatus::Paused)
            || matches!(self.status, TypingStatus::Resuming)
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

    pub fn check_completion(&mut self) -> bool {
        let typing_test_in_progress = self.is_typing() || self.is_resuming();
        if typing_test_in_progress && self.should_complete() {
            self.complete();
            return true;
        }
        false
    }

    fn is_previous_token_a_space(&self) -> bool {
        if let (Some(curr), Some(prev)) = (self.current_token(), self.prev_token()) {
            return curr.target != ' ' && prev.target == ' ';
        }
        false
    }

    fn prev_token(&self) -> Option<&Token> {
        if self.current_pos == 0 {
            return None;
        }
        self.tokens.get(self.current_pos - 1)
    }

    fn prev_word_has_extra_tokens(&self) -> bool {
        if self.current_pos == 0 {
            return false;
        }

        let mut pos = self.current_pos - 1;
        let mut found_space = false;
        while pos > 0 {
            if let Some(token) = self.tokens.get(pos) {
                if token.target == ' ' {
                    if found_space {
                        break;
                    } else {
                        found_space = true;
                    }
                } else if token.is_extra_token() {
                    return true;
                }
            }
            pos -= 1;
        }
        false
    }

    fn prev_word_has_skipped_tokens(&self) -> bool {
        if self.current_pos == 0 {
            return false;
        }

        let mut pos = self.current_pos - 1;
        while pos > 0 {
            if let Some(token) = self.tokens.get(pos) {
                if token.is_skipped {
                    return true;
                }
                if token.target == ' ' {
                    // space before the prev word
                    break;
                }
            }
            pos -= 1;
        }
        false
    }

    fn would_complete_word_at(&self, pos: usize) -> bool {
        if pos == 0 {
            return false;
        }
        // a word is `completed` if the previous token is a `<space>` or we are at the end of word
        self.tokens
            .get(pos - 1)
            .is_some_and(|token| token.target == ' ')
            || pos >= self.tokens.len()
    }

    /// Checks if the word at the given position contains errors or not
    #[inline]
    pub fn is_word_wrong(&self, pos: usize) -> bool {
        self.words
            .get(pos)
            .is_some_and(|w| w.completed && w.error_count > 0)
    }

    /// Gets the current token
    #[inline]
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current_pos)
    }

    /// Gets the previous word
    #[inline]
    fn prev_word(&self) -> Option<&Word> {
        if self.current_word_idx == 0 {
            return None;
        }
        self.words.get(self.current_word_idx - 1)
    }

    /// Mutably gets the current token
    #[inline]
    fn current_token_mut(&mut self) -> Option<&mut Token> {
        self.tokens.get_mut(self.current_pos)
    }

    /// Mutablly gets the current word
    #[inline]
    fn current_word_mut(&mut self) -> Option<&mut Word> {
        self.words.get_mut(self.current_word_idx)
    }

    /// Gets the current word
    #[inline]
    #[allow(dead_code)]
    fn current_word(&self) -> Option<&Word> {
        self.words.get(self.current_word_idx)
    }

    fn should_mark_word_as_completed(&self) -> bool {
        if self.current_pos == 0 {
            return false;
        }
        let curr_char = self.prev_token();
        let is_space_x = curr_char.map_or_else(|| false, |c| c.target == ' ');
        let is_end = self.current_pos >= self.tokens.len();

        is_space_x || is_end
    }

    // NOTE: i did this words end and start time because i think it would be nice to show in a
    // graph visualiation, but if this gets too annoying to deal with then remove it.
    fn mark_word_as_completed(&mut self) {
        if let Some(word) = self.current_word_mut() {
            word.completed = true;
            word.end_time = Some(Instant::now());
        }
        self.current_word_idx += 1;
        self.extra_errors_count = 0;

        if let Some(word) = self.current_word_mut() {
            word.start_time = Some(Instant::now())
        }
    }

    pub fn should_complete(&self) -> bool {
        // all words are typed, should end test
        if self.current_pos >= self.tokens.len() {
            return true;
        }

        match self.mode {
            Mode::Time(secs) => {
                if self.start_time.is_some() {
                    self.elapsed_time() >= Duration::from_secs(secs as u64)
                } else {
                    false
                }
            }
            Mode::Words(count) => self.current_word_idx >= count,
        }
    }

    pub fn complete(&mut self) {
        self.update_metrics();
        let completion_time = Instant::now();
        self.end_time = Some(completion_time);

        if let Some(word) = self.current_word_mut() {
            if !word.completed {
                word.completed = true;
                word.end_time = Some(completion_time);
            }
        }

        self.update_metrics();
        log_debug!("Wpm samples: {:?}", self.wpm_snapshots);

        self.status = TypingStatus::Completed;
    }

    fn is_at_word_start(&self) -> bool {
        self.current_pos == 0 || self.prev_token().is_some_and(|prev| prev.target == ' ')
    }

    /// Calculates the target position for a space jump
    fn calculate_space_jump_target(&self) -> Option<usize> {
        // bin search the next word boundary
        let next_boundary_idx = self
            .word_boundaries
            .partition_point(|&boundary| boundary <= self.current_pos);

        let next_boundary = *self.word_boundaries.get(next_boundary_idx)?;

        Some(next_boundary)
    }

    /// Activate the spacedrive and perform the space jump to the target location
    fn perform_space_jump(&mut self, target_pos: usize) -> Result<(), AppError> {
        let source_pos = self.current_pos;
        let jump_length = target_pos.saturating_sub(source_pos);

        if jump_length == 0 {
            return Ok(());
        }

        self.space_jump_stack
            .push(SpaceJump::new(source_pos, target_pos));

        self.total_errors += jump_length;

        // fill the offset with spaces
        let spaces: String = " ".repeat(jump_length);
        self.typed_text.push_str(&spaces);

        // update the offset tokens and words states
        for pos in source_pos..target_pos {
            self.current_pos = pos;

            if let Some(token) = self.current_token_mut() {
                token.typed = Some(' ');
                token.typed_at = Some(Instant::now());
                token.is_wrong = true;
                token.is_skipped = true;
            }
        }

        // update error count for the currently skipped word
        if let Some(word) = self.current_word_mut() {
            word.error_count += jump_length;
        }

        self.current_pos = target_pos;

        // did the jump complted a word
        if self.should_mark_word_as_completed() {
            self.mark_word_as_completed();
        }

        Ok(())
    }

    /// Reverses a given space jump operation
    fn undo_space_jump(&mut self, jump: SpaceJump) -> Result<(), AppError> {
        let positions_to_undo = jump.target_pos.saturating_sub(jump.source_pos);

        self.total_errors = self.total_errors.saturating_sub(positions_to_undo);

        // remove the spaces added by the space jump
        let new_len = self.typed_text.len().saturating_sub(positions_to_undo);
        self.typed_text.truncate(new_len);

        let word_was_completed = self.would_complete_word_at(jump.target_pos);

        // if a word was completed during the space jump... unmark it and go back one word
        if word_was_completed && self.current_word_idx > 0 {
            self.current_word_idx -= 1;
            if let Some(word) = self.current_word_mut() {
                word.completed = false;
                word.end_time = None;
            }
        }

        // decrese the error count of the space jumped word equivalent to the undid positions
        if word_was_completed {
            if let Some(word) = self.current_word_mut() {
                word.error_count = word.error_count.saturating_sub(positions_to_undo);
            }
        }

        // restore the space jumped token state
        for pos in (jump.source_pos..jump.target_pos).rev() {
            if let Some(token) = self.tokens.get_mut(pos) {
                token.typed = None;
                token.typed_at = None;
                token.is_wrong = false;
                token.is_skipped = false;
            }
        }

        self.current_pos = jump.source_pos;
        self.space_jump_stack.pop();

        Ok(())
    }

    /// Returns an iterator over all words with their statistics
    pub fn words_iter(&self) -> impl Iterator<Item = &Word> {
        self.words.iter()
    }

    /// Returns the current WPM
    pub fn wpm(&mut self) -> f64 {
        self.metrics.wpm.unwrap_or(0.0)
    }

    /// Returns the current WPS (Words Per Second)
    pub fn wps(&mut self) -> f64 {
        self.wpm() / 60.0
    }

    /// Returns the current accuracy as a percentage (0.0 to 1.0)
    pub fn accuracy(&mut self) -> f64 {
        self.metrics.accuracy.unwrap_or(0.0)
    }

    pub fn consistency(&mut self) -> f64 {
        self.metrics.consistency.unwrap_or(0.0)
    }

    /// Returns a summary of the current typing session
    pub fn summary(&self) -> Summary {
        Summary {
            wpm: self.metrics.wpm.unwrap_or(0.0),
            wps: self.metrics.wpm.unwrap_or(0.0) / 60.0,
            snapshots: self.wpm_snapshots.clone(),
            accuracy: self.metrics.accuracy.unwrap_or(0.0),
            consistency: self.metrics.consistency.unwrap_or(0.0),
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
            Mode::Words(_) => (self.current_pos as f64 / self.text.len() as f64).min(1.0),
            Mode::Time(total_seconds) => {
                if self.status == TypingStatus::Completed {
                    1.0
                } else if let Some(start) = self.start_time {
                    let elapsed = start.elapsed().as_secs_f64();
                    (elapsed / total_seconds as f64).min(1.0)
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

    /// Returns the number of correctly typed non-space characters so far
    pub fn correct_non_space_chars_count(&self) -> usize {
        self.tokens
            .iter()
            .take(self.current_pos)
            .filter(|token| token.is_correct_non_space_token())
            .count()
    }

    pub fn try_metrics_update(&mut self) {
        if !self.is_typing() || self.should_complete() {
            return;
        }

        let now = Instant::now();

        // store snapshot of current wpm every elapsed second while typing
        if self.should_snapshot_wpm(now) {
            let current_wpm = self.calculate_wpm();
            if current_wpm > 0.0 {
                self.wpm_snapshots.push(current_wpm);
            }
            self.last_snapshot_time = Some(now);
        }

        if self.should_update_metrics(now) {
            self.update_metrics();
            self.metrics.last_updated_at = Some(now);
        }
    }

    fn should_snapshot_wpm(&self, now: Instant) -> bool {
        let elapsed = self.elapsed_time();
        elapsed >= Duration::from_secs(1)
            && self
                .last_snapshot_time
                .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(1))
    }

    fn should_update_metrics(&self, now: Instant) -> bool {
        self.metrics
            .last_updated_at
            .is_none_or(|last_update| now.duration_since(last_update) >= Duration::from_secs(1))
    }

    pub fn update_metrics(&mut self) {
        self.metrics.wpm = Some(self.calculate_wpm());
        self.metrics.accuracy = Some(self.calculate_accuracy());
        self.metrics.consistency = Some(self.calculate_consistency());
    }

    fn calculate_wpm(&self) -> f64 {
        // anti keyboard smash tactic
        if self.correct_non_space_chars_count() == 0 {
            return 0.0;
        }
        let correct_chars_typed = self.correct_chars_count() as f64;
        if correct_chars_typed <= 0.0 {
            return 0.0;
        }

        let elapsed_secs = self.elapsed_time().as_secs_f64();
        let effective_secs = elapsed_secs.max(0.1); // avoid spikes on short time/word tests

        // wpm = (chars / 5) per minute
        (correct_chars_typed / 5.0) / (effective_secs / 60.0)
    }

    fn calculate_accuracy(&self) -> f64 {
        let total_typed = self.typed_text.len() as f64;
        if total_typed > 0.0 {
            self.correct_chars_count() as f64 / total_typed
        } else {
            0.0
        }
    }

    fn calculate_consistency(&self) -> f64 {
        let mean = self.wpm_snapshots.mean();
        if mean == 0.0 || self.wpm_snapshots.is_empty() {
            0.0
        } else {
            let std = self.wpm_snapshots.std_dev();
            (100.0 - (std / mean) * 100.0).max(0.0)
        }
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
    pub snapshots: WpmSnapshots,
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
    pub fn raw_wpm(&self) -> f64 {
        // basically raw wpm
        let total_typed = (self.correct_chars + self.total_errors) as f64;
        if total_typed <= 0.0 {
            return 0.0;
        }
        let elapsed_secs = self.elapsed_time.as_secs_f64();
        let effective_secs = elapsed_secs.max(0.1);
        (total_typed / 5.0) / (effective_secs / 60.0)
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

/// Snapshots of WPM sample taken periodically while typing
#[derive(Debug, Clone, Default)]
pub struct WpmSnapshots {
    snapshots: Vec<f64>,
}

impl WpmSnapshots {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn push(&mut self, wpm: f64) {
        self.snapshots.push(wpm);
    }

    /// Returns an iterator over the WPM snapshots
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        self.snapshots.iter()
    }

    #[inline]
    pub fn min(&self) -> f64 {
        self.snapshots
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    #[inline]
    pub fn max(&self) -> f64 {
        self.snapshots
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    #[inline]
    pub fn mean(&self) -> f64 {
        if self.snapshots.is_empty() {
            0.0
        } else {
            self.snapshots.iter().sum::<f64>() / self.snapshots.len() as f64
        }
    }

    /// Calculates the standard deviation based on the snapshots
    pub fn std_dev(&self) -> f64 {
        if self.snapshots.len() < 2 {
            return 0.0;
        }

        let mean: f64 = self.snapshots.iter().sum::<f64>() / self.snapshots.len() as f64;
        let variance: f64 = self
            .snapshots
            .iter()
            .map(|wpm| (wpm - mean).powi(2))
            .sum::<f64>()
            / self.snapshots.len() as f64;

        variance.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        tracker.type_char('c').unwrap();
        assert_eq!(tracker.status, TypingStatus::InProgress);
        tracker.toggle_pause();
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

        assert!(tracker.check_completion());
        assert_eq!(tracker.status, TypingStatus::Completed);
    }

    #[test]
    fn test_word_mode_completion() {
        let mut tracker = Tracker::new("hello world".to_string(), Mode::with_words(2));
        tracker.start_typing();
        for c in "hello ".chars() {
            tracker.type_char(c).unwrap()
        }

        assert!(!tracker.check_completion());

        for c in "world".chars() {
            tracker.type_char(c).unwrap()
        }

        assert!(tracker.check_completion())
    }

    #[test]
    fn test_progress_words_mode() {
        let text = "testing termitype";
        let mut tracker = Tracker::new(text.to_string(), Mode::Words(10));
        tracker.current_pos = 5;
        assert_eq!(tracker.progress(), 5.0 / text.len() as f64);

        tracker.current_pos = text.len();
        assert_eq!(tracker.progress(), 1.0);

        let empty_tracker = Tracker::new("".to_string(), Mode::Words(10));
        assert_eq!(empty_tracker.progress(), 1.0);
    }

    #[test]
    fn test_progress_time_mode() {
        let total_seconds = 10;
        let mut tracker = Tracker::new("test".to_string(), Mode::Time(total_seconds));

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
        let mut tracker = Tracker::new(str.clone(), Mode::with_time(30));
        tracker.start_typing();

        for c in str.chars() {
            tracker.type_char(c).unwrap();
        }

        tracker.start_time = Some(Instant::now() - Duration::from_secs(15));
        tracker.check_completion(); // Complete the test to update metrics

        let summary = tracker.summary();
        assert!(summary.wpm >= 0.0);
        assert!(summary.wps >= 0.0);
        assert!(summary.accuracy > 0.0);
        assert_eq!(summary.total_chars, str.len());
        assert_eq!(summary.correct_chars, str.len());
        assert_eq!(summary.total_errors, 0);
        assert!(summary.elapsed_time >= Duration::from_secs(10));
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
        assert_eq!(tracker.current_word_idx, 0);
        tracker.type_char('h').unwrap();
        tracker.type_char('i').unwrap();
        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_word_idx, 1);
        assert_eq!(tracker.current_pos, 3);
        tracker.type_char('y').unwrap();
        tracker.type_char('o').unwrap();
        assert_eq!(tracker.current_word_idx, 1);
        assert_eq!(tracker.current_pos, 5);
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

        assert!(tracker.check_completion());
        let summary = tracker.summary();
        assert_eq!(tracker.status, TypingStatus::Completed);
        assert!(summary.wpm > 0.0);

        // 4 chars = 0.8 words, in 0.1 seconds = 0.8 / (0.1/60) = 480 WPM
        assert_eq!(summary.wpm, 480.0);
    }

    #[test]
    fn test_typing_wrong_char_on_word_boundary_should_add_extra_token() {
        let mut tracker = Tracker::new("another test".to_string(), Mode::with_words(2));
        let word_to_type = "another";
        for c in word_to_type.chars() {
            tracker.type_char(c).unwrap();
        }

        assert_eq!(tracker.current_pos, word_to_type.len());
        assert_eq!(tracker.correct_chars_count(), word_to_type.len());

        // wrong token at boundary, add it
        tracker.type_char('W').unwrap();
        assert_eq!(tracker.current_pos, word_to_type.len() + 1);
        assert_eq!(tracker.correct_chars_count(), word_to_type.len());
    }

    #[test]
    fn test_fast_wrong_input_keeps_wpm_low() {
        let target_text = "this is a longer piece of reference words for testing".to_string();
        let mut tracker = Tracker::new(target_text.clone(), Mode::with_time(60));

        for _ in 0..target_text.len() - 1 {
            if tracker.check_completion() {
                break;
            }
            for _ in 0..6 {
                if tracker.type_char('x').is_err() {
                    break;
                }
            }
            let _ = tracker.type_char(' ');
        }

        let wpm = tracker.wpm();
        assert_eq!(wpm, 0.0, "expected 0 wpm, got {wpm} wpm");

        let mut tracker = Tracker::new(target_text.clone(), Mode::with_words(target_text.len()));

        for _ in 0..target_text.len() - 1 {
            if tracker.check_completion() {
                break;
            }
            for _ in 0..6 {
                if tracker.type_char('x').is_err() {
                    break;
                }
            }
            let _ = tracker.type_char(' ');
        }

        let wpm = tracker.wpm();
        assert_eq!(wpm, 0.0, "expected 0 wpm, got {wpm} wpm");
    }

    #[test]
    fn test_disallow_backspace_at_boundary_after_correct_word() {
        let mut tracker = Tracker::new("termitype FAIL another".to_string(), Mode::with_words(3));

        let str = "termitype";
        for c in str.chars() {
            tracker.type_char(c).unwrap();
        }

        tracker.type_char(' ').unwrap();

        assert_eq!(tracker.current_pos, 10);
        assert_eq!(tracker.current_token().unwrap().target, 'F');

        tracker.backspace().unwrap(); // we should remain on the same spot

        assert_eq!(tracker.current_pos, 10);
        assert_eq!(tracker.current_token().unwrap().target, 'F');

        tracker.type_char('F').unwrap();
        assert_eq!(tracker.current_pos, 11);
        assert_eq!(tracker.current_token().unwrap().target, 'A');

        tracker.backspace().unwrap();

        assert_eq!(tracker.current_pos, 10);
        assert_eq!(tracker.current_token().unwrap().target, 'F');

        // NOTE(ema): the disallow rule only applies if the previous word is not fully correct

        // typo kind
        for c in "FsIL".chars() {
            tracker.type_char(c).unwrap();
        }
        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_pos, 15);
        assert_eq!(tracker.current_word_idx, 2); // another
        assert_eq!(tracker.current_token().unwrap().target, 'a');

        tracker.backspace().unwrap();
        tracker.backspace().unwrap();

        assert_eq!(tracker.current_pos, 13);
        assert_eq!(tracker.current_word_idx, 1); // FAIL
        assert_eq!(tracker.current_token().unwrap().target, 'L');
    }

    #[test]
    fn test_space_beyond_first_token_should_skip_to_next_word() {
        let mut tracker = Tracker::new("hello there test".to_string(), Mode::with_words(3));

        tracker.type_char('h').unwrap();
        assert_eq!(tracker.current_pos, 1);
        assert_eq!(tracker.current_word_idx, 0);
        assert_eq!(tracker.current_word().unwrap().target, "hello".to_string());
        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_pos, 6);
        assert_eq!(tracker.current_word_idx, 1);
        assert_eq!(tracker.current_word().unwrap().target, "there".to_string());

        tracker.type_char('t').unwrap(); // there
        assert_eq!(tracker.current_pos, 7);
        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_pos, 12);
        assert_eq!(tracker.current_word_idx, 2);
        assert_eq!(tracker.current_word().unwrap().target, "test".to_string());
    }

    #[test]
    fn test_jump_space_to_start_of_next_word() {
        let mut tracker = Tracker::new("another space jump test".to_string(), Mode::with_words(4));

        tracker.type_char('f').unwrap();
        assert_eq!(tracker.current_pos, 1);
        assert_eq!(tracker.current_word_idx, 0);

        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_pos, 8);
        assert_eq!(tracker.current_word_idx, 1);
        assert_eq!(tracker.current_word().unwrap().error_count, 0);

        tracker.type_char('f').unwrap();
        assert_eq!(tracker.current_pos, 9);
        assert_eq!(tracker.current_word_idx, 1);

        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_pos, 14);
        assert_eq!(tracker.current_word_idx, 2);
        assert_eq!(tracker.current_word().unwrap().error_count, 0);

        tracker.type_char('f').unwrap();
        assert_eq!(tracker.current_pos, 15);
        assert_eq!(tracker.current_word_idx, 2);
        assert_eq!(tracker.current_word().unwrap().error_count, 1);
    }

    #[test]
    fn test_space_in_boundary_shouldnt_skip() {
        let mut tracker = Tracker::new("hello world".to_string(), Mode::default());
        for c in "hello".chars() {
            tracker.type_char(c).unwrap();
        }
        assert_eq!(tracker.current_pos, 5);
        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_pos, 6);
        assert!(tracker.type_char(' ').is_err());
        assert_eq!(tracker.current_pos, 6);
    }

    #[test]
    fn test_space_jump_and_back_with_fixed_error() {
        let mut tracker =
            Tracker::new("another change to do here".to_string(), Mode::with_words(5));

        tracker.type_char('a').unwrap();
        tracker.type_char(' ').unwrap();
        tracker.backspace().unwrap();
        tracker.type_char('a').unwrap();
        tracker.backspace().unwrap();

        // complete the word
        for c in "nother".chars() {
            tracker.type_char(c).unwrap();
        }
        tracker.type_char(' ').unwrap();

        // first word is corrected and shouldn't be marked as wrong
        assert_eq!(tracker.words.first().unwrap().error_count, 0);
    }

    #[test]
    fn test_backspace_after_multiple_chained_spaces() {
        let target_text = "hello there termitype hope you doing good".to_string();
        let mut tracker = Tracker::new(target_text, Mode::with_time(30));
        tracker.type_char('h').unwrap();
        assert_eq!(tracker.current_pos, 1);
        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_pos, 6); // just before "there"
        tracker.type_char('t').unwrap();
        assert_eq!(tracker.current_pos, 7);
        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_pos, 12); // just before "termitype"
        tracker.backspace().unwrap();
        assert_eq!(tracker.current_pos, 7);
        tracker.backspace().unwrap();
        assert_eq!(tracker.current_pos, 6);
        tracker.backspace().unwrap();
        assert_eq!(tracker.current_pos, 1);
    }

    #[test]
    fn test_space_jump_long_word_with_offsets() {
        let text = "supercalifragilisticexpialidocious another".to_string();
        let long_word_len = "supercalifragilisticexpialidocious".len(); // 34
        let next_word_start = long_word_len + 1; // 35

        let offsets = [1, 5, 10, 20, 30];

        for &offset in &offsets {
            let mut tracker = Tracker::new(text.clone(), Mode::with_words(2));

            for c in text.chars().take(offset) {
                tracker.type_char(c).unwrap();
            }
            assert_eq!(tracker.current_pos, offset);
            assert_eq!(tracker.current_word_idx, 0);

            tracker.type_char(' ').unwrap();

            assert_eq!(tracker.current_pos, next_word_start);
            assert_eq!(tracker.current_word_idx, 1);
            assert_eq!(tracker.current_word().unwrap().target, "another");

            assert_eq!(tracker.typed_text.len(), next_word_start);
            assert!(tracker.typed_text[offset..].chars().all(|c| c == ' '));
        }
    }

    #[test]
    fn test_words_mode_completion_with_extra_tokens_at_boundary() {
        let mut tracker = Tracker::new("hello world test".to_string(), Mode::with_words(3));
        tracker.start_typing();

        for c in "hello".chars() {
            tracker.type_char(c).unwrap();
        }
        assert_eq!(tracker.current_word_idx, 0);
        assert!(!tracker.check_completion());

        // wrong tokens, should add extra tokens
        for c in "xxxyyyzzz".chars() {
            tracker.type_char(c).unwrap();
        }

        // we still at word `0`, not done yet
        assert_eq!(tracker.current_word_idx, 0);
        assert!(!tracker.check_completion());

        // move to next word
        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_word_idx, 1);
        assert!(!tracker.check_completion());

        for c in "world".chars() {
            tracker.type_char(c).unwrap();
        }
        assert!(!tracker.check_completion()); // we still in word_idx=1 out of 2, not done yet

        tracker.type_char(' ').unwrap();

        // word_idx=2, but we haven't completed the word so we are not done
        assert!(!tracker.check_completion());
        assert_eq!(tracker.current_word_idx, 2);

        for c in "test".chars() {
            tracker.type_char(c).unwrap();
        }

        assert!(tracker.check_completion());
    }

    #[test]
    fn test_consistency_perfect() {
        let mut tracker = Tracker::new("test".to_string(), Mode::with_time(60));
        // Simulate perfect consistency: all WPM samples the same
        tracker.wpm_snapshots.snapshots = vec![60.0, 60.0, 60.0, 60.0];
        tracker.update_metrics();
        assert_eq!(tracker.consistency(), 100.0);
    }

    #[test]
    fn test_consistency_varying() {
        let mut tracker = Tracker::new("test".to_string(), Mode::with_time(60));
        // Simulate varying WPM: mean 60, std dev 10
        tracker.wpm_snapshots.snapshots = vec![50.0, 70.0];
        tracker.update_metrics();
        let expected = 100.0 - (10.0 / 60.0) * 100.0; // 83.333...
        assert!((tracker.consistency() - expected).abs() < 0.001);
    }

    #[test]
    fn test_consistency_high_variation() {
        let mut tracker = Tracker::new("test".to_string(), Mode::with_time(60));
        // High variation: mean 60, std dev 60
        tracker.wpm_snapshots.snapshots = vec![0.0, 120.0];
        tracker.update_metrics();
        assert_eq!(tracker.consistency(), 0.0);
    }

    #[test]
    fn test_consistency_no_samples() {
        let mut tracker = Tracker::new("test".to_string(), Mode::with_time(60));
        // No samples
        tracker.wpm_snapshots.snapshots = vec![];
        tracker.update_metrics();
        assert_eq!(tracker.consistency(), 0.0);
    }

    #[test]
    fn test_consistency_zero_mean() {
        let mut tracker = Tracker::new("test".to_string(), Mode::with_time(60));
        // All zero WPM
        tracker.wpm_snapshots.snapshots = vec![0.0, 0.0];
        tracker.update_metrics();
        assert_eq!(tracker.consistency(), 0.0);
    }

    #[test]
    fn test_space_jump_skipped_words_no_error_count() {
        let mut tracker = Tracker::new("hello world test".to_string(), Mode::with_words(3));
        tracker.start_typing();

        tracker.type_char('h').unwrap();
        assert_eq!(tracker.current_pos, 1);
        assert_eq!(tracker.words[0].error_count, 0);

        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_pos, 6);
        assert_eq!(tracker.current_word_idx, 1);

        // NOTE: we count the space as wrong even tho we don't render it in the ui, hence the `5`
        assert_eq!(tracker.words[0].error_count, 5);
        assert!(tracker.is_word_wrong(0));

        assert_eq!(tracker.words[1].error_count, 0);
    }

    #[test]
    fn test_backspace_extra_tokens_at_boundary_should_fix_error_count() {
        let mut tracker = Tracker::new("hello world".to_string(), Mode::with_words(2));
        tracker.start_typing();

        for c in "hello".chars() {
            tracker.type_char(c).unwrap();
        }
        assert_eq!(tracker.current_pos, 5);
        assert_eq!(tracker.words[0].error_count, 0);
        assert_eq!(tracker.total_errors, 0);

        tracker.type_char('X').unwrap();
        tracker.type_char('Y').unwrap();
        tracker.type_char('Z').unwrap();

        tracker.backspace().unwrap();
        tracker.backspace().unwrap();
        tracker.backspace().unwrap();

        assert_eq!(tracker.current_pos, 5);
        assert_eq!(tracker.words[0].error_count, 0);
        assert_eq!(tracker.total_errors, 0);

        tracker.type_char(' ').unwrap();
        assert_eq!(tracker.current_word_idx, 1);

        assert!(tracker.words[0].completed);
        assert_eq!(
            tracker.words[0].error_count, 0,
            "Completed word should have 0 errors after correction"
        );
    }

    #[test]
    fn test_complete_word_with_errors_then_backspace_and_fix() {
        let mut tracker = Tracker::new("hello world".to_string(), Mode::with_words(2));
        tracker.start_typing();

        tracker.type_char('h').unwrap();
        tracker.type_char('a').unwrap(); // wrong char!
        tracker.type_char('l').unwrap();
        tracker.type_char('l').unwrap();
        tracker.type_char('o').unwrap();
        assert_eq!(tracker.words[0].error_count, 1);

        tracker.type_char(' ').unwrap();
        assert!(tracker.words[0].completed);
        assert_eq!(tracker.words[0].error_count, 1);
        assert_eq!(tracker.current_word_idx, 1);

        tracker.backspace().unwrap();
        assert!(!tracker.words[0].completed);
        assert_eq!(tracker.current_word_idx, 0);

        tracker.backspace().unwrap();
        tracker.backspace().unwrap();
        tracker.backspace().unwrap();
        tracker.backspace().unwrap();
        assert_eq!(tracker.words[0].error_count, 0);

        tracker.type_char('e').unwrap();
        tracker.type_char('l').unwrap();
        tracker.type_char('l').unwrap();
        tracker.type_char('o').unwrap();
        assert_eq!(tracker.words[0].error_count, 0);

        tracker.type_char(' ').unwrap();
        assert!(tracker.words[0].completed);
        assert_eq!(tracker.words[0].error_count, 0);
    }

    #[test]
    fn test_word_with_errors_then_space_jump_then_backspace_and_fix() {
        let mut tracker = Tracker::new("hello world test".to_string(), Mode::with_words(3));
        tracker.start_typing();

        tracker.type_char('h').unwrap();
        tracker.type_char('a').unwrap(); // wrong char
        assert_eq!(tracker.words[0].error_count, 1);

        tracker.type_char(' ').unwrap();
        assert!(tracker.words[0].completed);
        assert_eq!(tracker.words[0].error_count, 5);
        assert!(tracker.is_word_wrong(0));
        assert_eq!(tracker.current_word_idx, 1);
        assert_eq!(tracker.current_pos, 6);

        tracker.backspace().unwrap();
        assert_eq!(tracker.current_pos, 2);

        assert_eq!(tracker.current_word_idx, 0);
        assert!(!tracker.words[0].completed,);

        tracker.backspace().unwrap();
        assert_eq!(tracker.words[0].error_count, 0);

        tracker.type_char('e').unwrap();
        tracker.type_char('l').unwrap();
        tracker.type_char('l').unwrap();
        tracker.type_char('o').unwrap();
        tracker.type_char(' ').unwrap();

        assert_eq!(tracker.words[0].error_count, 0);
        assert!(tracker.words[0].completed);
    }

    #[test]
    fn test_bug_reproduction_wrong_words_completed() {
        let text = "hello world test another text to test against".to_string();
        let mut tracker = Tracker::new(text, Mode::with_words(8));
        tracker.start_typing();

        for c in "hello worlff".chars() {
            tracker.type_char(c).unwrap();
        }

        tracker.backspace().unwrap();
        tracker.backspace().unwrap();

        for c in "d test ano".chars() {
            tracker.type_char(c).unwrap();
        }

        assert!(tracker.words[0].completed);
        assert!(!tracker.is_word_wrong(0));

        assert!(tracker.words[1].completed);
        assert!(!tracker.is_word_wrong(1));

        assert!(tracker.words[2].completed);
        assert!(!tracker.is_word_wrong(2));
    }

    #[test]
    fn test_allow_backspace_if_prev_word_is_correct_but_has_extra_chars() {
        let text = "hello another test".to_string();
        let mut tracker = Tracker::new(text, Mode::with_words(3));
        tracker.start_typing();

        for c in "helloFF".chars() {
            tracker.type_char(c).unwrap();
        }

        tracker.type_char(' ').unwrap();

        // at this point we have the `extra chars` and the word is typed correctly
        assert_eq!(tracker.current_word().unwrap().target, "another");

        tracker.backspace().unwrap();
        assert_eq!(tracker.current_word().unwrap().target, "hello"); // we should be able to go back
    }

    #[test]
    fn test_space_jump_should_mark_word_as_wrong() {
        let text = "hello another test".to_string();
        let mut tracker = Tracker::new(text, Mode::with_words(3));
        tracker.start_typing();
        tracker.type_char('h').unwrap();
        tracker.type_char(' ').unwrap();

        assert!(tracker.is_word_wrong(0));
    }

    #[test]
    fn test_should_be_able_to_backspace_after_a_pause() {
        let text = "correct word".to_string();
        let mut tracker = Tracker::new(text, Mode::with_words(2));
        tracker.start_typing();

        tracker.type_char('t').unwrap();
        tracker.type_char('e').unwrap();
        tracker.type_char('s').unwrap();
        tracker.type_char('t').unwrap();
        assert_eq!(tracker.current_pos, 4);
        tracker.backspace().unwrap();
        assert_eq!(tracker.current_pos, 3);

        tracker.toggle_pause();
        tracker.toggle_pause();
        tracker.backspace().unwrap();
        assert_eq!(tracker.current_pos, 2);
    }
}
