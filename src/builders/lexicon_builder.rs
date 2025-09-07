use std::{collections::HashMap, sync::OnceLock};

use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    assets,
    config::Config,
    constants::{DEFAULT_LANGUAGE, WPS_TARGET},
    error::AppError,
};

const SYMBOLS: &[char] = &[
    '@', '#', '$', '%', '&', '*', '(', ')', '+', '-', '/', '=', '?', '<', '>', '^', '_', '`', '{',
    '|', '}', '~',
];
const PUNCTUATION: &[char] = &['.', ',', '!', '?', ';', ':'];
const NUMBERS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const DEFAULT_LEXICON: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "I", "it", "for", "not", "on",
    "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we", "say",
    "her", "she", "or", "an", "will", "my", "one", "all", "would", "there", "their", "what", "so",
    "up", "out", "if", "about", "who", "get", "which", "go", "me",
];

const SYMBOL_PROBABILITY: f64 = 0.20;
const PUNCTUATION_PROBABILITY: f64 = 0.30;
const NUMBER_PROBABILITY: f64 = 0.15;

#[derive(Debug, Serialize, Deserialize)]
struct Language {
    name: String,
    words: Vec<String>,
}

#[derive(Debug)]
pub struct Lexicon {
    pub words: Vec<String>,
    builder: LexiconBuilder,
}

impl Lexicon {
    /// Creates a new Lexicon with words generated from the config.
    pub fn new(config: &Config) -> Result<Self, AppError> {
        let mut builder = LexiconBuilder::new();
        let words = builder.generate_test(config)?;
        Ok(Self { words, builder })
    }

    /// Regenerates the lexicon composition.
    pub fn regenerate(&mut self, config: &Config) -> Result<(), AppError> {
        self.words = self.builder.generate_test(config)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LexiconBuilder {
    languages: HashMap<String, Vec<String>>,
    shuffled_pools: HashMap<String, Vec<usize>>,
    rng: rand::rngs::ThreadRng,
}

impl LexiconBuilder {
    /// Creates a new LexiconBuilder instance.
    pub fn new() -> Self {
        let mut builder = Self {
            languages: HashMap::new(),
            shuffled_pools: HashMap::new(),
            rng: rand::rng(),
        };
        if builder.load_language(DEFAULT_LANGUAGE).is_err() {
            Self::add_default_words(&mut builder);
        }
        builder
    }

    /// Generates the lexicon used in the typing test.
    pub fn generate_test(&mut self, config: &Config) -> Result<Vec<String>, AppError> {
        // NOTE: im sure we can optimize the sh*t out of this, but good enough for now.
        // TODO: when custom words get implemented take it into consideration here
        if let Some(custom_words) = &config.cli.words {
            return Ok(custom_words
                .split_whitespace()
                .map(String::from)
                .collect::<Vec<String>>());
        }
        let lang = config.current_language();
        self.ensure_language_loaded(&lang)?;

        let words = &self.languages[&lang];
        let shuffled_idxs = &self.shuffled_pools[&lang];

        // if we are on time mode, we must ensure we genearate enough words even for mythicalrocket
        let word_count = if config.current_mode().is_time_mode() {
            config.current_mode().value() * WPS_TARGET
        } else {
            config.current_mode().value()
        };

        let mut selected_words: Vec<&str> = (0..word_count)
            .map(|i| words[shuffled_idxs[i % shuffled_idxs.len()]].as_str())
            .collect();

        // re-shuffle
        selected_words.shuffle(&mut self.rng);

        Self::prevent_consecutive_duplicates(&mut selected_words);

        // add extras such as punctuation, symbols, numbers, etc.
        let extras: Vec<Option<char>> = (0..word_count)
            .map(|_| {
                if config.using_symbols() && self.rng.random_bool(SYMBOL_PROBABILITY) {
                    Some(SYMBOLS[self.rng.random_range(0..SYMBOLS.len())])
                } else if config.using_punctuation()
                    && self.rng.random_bool(PUNCTUATION_PROBABILITY)
                {
                    Some(PUNCTUATION[self.rng.random_range(0..PUNCTUATION.len())])
                } else if config.using_numbers() && self.rng.random_bool(NUMBER_PROBABILITY) {
                    Some(NUMBERS[self.rng.random_range(0..NUMBERS.len())])
                } else {
                    None
                }
            })
            .collect();

        // build the actual lexicon
        let mut lexicon: Vec<String> = Vec::with_capacity(word_count);
        for (i, word) in selected_words.iter().enumerate() {
            let mut new_word = word.to_string();
            if let Some(extra) = extras[i] {
                new_word.push(extra);
            }
            lexicon.push(new_word);
        }

        Ok(lexicon)
    }

    /// Ensure we at least build from the default words dictionary
    fn add_default_words(builder: &mut Self) {
        let words = DEFAULT_LEXICON
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let mut idxs: Vec<usize> = (0..words.len()).collect();
        idxs.shuffle(&mut builder.rng);
        builder
            .languages
            .insert(DEFAULT_LANGUAGE.to_string(), words);
        builder
            .shuffled_pools
            .insert(DEFAULT_LANGUAGE.to_string(), idxs);
    }

    /// Ensures the language is loaded, load it if itsn't loaded already
    fn ensure_language_loaded(&mut self, lang: &str) -> Result<(), AppError> {
        if !self.languages.contains_key(lang) {
            self.load_language(lang)?;
        }
        Ok(())
    }

    /// Loads the given language.
    fn load_language(&mut self, lang: &str) -> Result<(), AppError> {
        if !Self::has_language(lang) {
            return Err(AppError::InvalidLanguage(lang.to_string()));
        }

        let content = assets::get_language(lang)
            .ok_or_else(|| AppError::Other(format!("Language not found: {}", lang)))?;

        let language: Language = serde_json::from_str(&content)?;
        let mut idxs: Vec<usize> = (0..language.words.len()).collect();
        idxs.shuffle(&mut self.rng);

        self.languages.insert(language.name.clone(), language.words);
        self.shuffled_pools.insert(language.name, idxs);
        Ok(())
    }

    /// Prevents back to back duplicated words
    fn prevent_consecutive_duplicates(words: &mut [&str]) {
        for i in 1..words.len() {
            if words[i] == words[i - 1] {
                let start = i + 1;
                let end = (start + 10).min(words.len()); // dont over do it, 10 words check is fine
                for j in start..end {
                    if words[j] != words[i] {
                        words.swap(i, j);
                        break;
                    }
                }
            }
        }
    }

    /// Returns the list of available languages.
    pub fn available_languages() -> &'static [String] {
        static LANGUAGES: OnceLock<Vec<String>> = OnceLock::new();
        LANGUAGES.get_or_init(assets::list_languages)
    }

    /// Checks if the given language is available.
    pub fn has_language(language: &str) -> bool {
        Self::available_languages()
            .iter()
            .any(|lang| lang == language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{Config, Mode},
        constants::MAX_CUSTOM_TIME,
    };

    fn create_builder() -> LexiconBuilder {
        LexiconBuilder::new()
    }

    #[test]
    fn test_default_builder_state() {
        let builder = create_builder();
        assert!(!builder.languages.is_empty());
        assert!(builder.languages.contains_key(DEFAULT_LANGUAGE));
    }

    #[test]
    fn test_language_loading() {
        let mut builder = create_builder();
        assert!(builder.languages.contains_key(DEFAULT_LANGUAGE));
        builder
            .load_language("spanish")
            .expect("Failed to load spanish");
        assert!(builder.languages.contains_key("spanish"));
    }

    #[test]
    fn test_invalid_language() {
        let mut builder = create_builder();
        let result = builder.load_language("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_back_to_back_duplicates() {
        let mut builder = create_builder();
        let mut config = Config::default();
        config
            .change_mode(crate::config::Mode::with_words(50))
            .unwrap();

        let test = builder.generate_test(&config).unwrap();

        for i in 1..test.len() {
            assert_ne!(
                test[i],
                test[i - 1],
                "Found consecutive duplicate at index {}",
                i
            );
        }
    }

    #[test]
    fn test_word_count() {
        let mut builder = create_builder();
        let mut config = Config::default();
        let count = 10;
        config
            .change_mode(crate::config::Mode::with_words(count))
            .unwrap();

        let test = builder.generate_test(&config).unwrap();
        assert_eq!(test.len(), count);
    }

    #[test]
    fn test_time_mode() {
        let mut builder = create_builder();
        let mut config = Config::default();
        let seconds_arr: [usize; 5] = [1, 10, 60, 120, MAX_CUSTOM_TIME];
        // do we have enough words for all the possible time mode ranges, even for fast typers?
        for seconds in seconds_arr {
            config.change_mode(Mode::with_time(seconds)).unwrap();
            let test = builder.generate_test(&config).unwrap();
            assert!(test.len() >= WPS_TARGET * seconds);
        }
    }
}
