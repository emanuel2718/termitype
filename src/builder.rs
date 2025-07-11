use std::{collections::HashMap, sync::OnceLock};

use rand::{seq::IndexedRandom, Rng};
use serde::{Deserialize, Serialize};

use crate::{assets, config::Config, constants::DEFAULT_LANGUAGE, log_debug};

const SYMBOLS: &[char] = &[
    '@', '#', '$', '%', '&', '*', '(', ')', '+', '-', '/', '=', '?', '<', '>', '^', '_', '`', '{',
    '|', '}', '~',
];
const PUNCTUATION: &[char] = &['.', ',', '!', '?', ';', ':'];
const NUMBERS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const DEFAULT_WORDS: &[&str] = &["the", "be", "to", "of", "and"];

const SYMBOL_PROBABILITY: f64 = 0.20;
const PUNCTUATION_PROBABILITY: f64 = 0.30;
const NUMBER_PROBABILITY: f64 = 0.15;

#[derive(Debug, Serialize, Deserialize)]
struct Language {
    name: String,
    words: Vec<String>,
}

#[derive(Debug)]
pub struct Builder {
    languages: HashMap<String, Vec<String>>,
    rng_cache: rand::rngs::ThreadRng,
    result_cache: String,
    words_cache: Vec<String>,
}

impl Builder {
    /// Creates a new Builder instance with the give language.
    pub fn new() -> Self {
        let mut builder = Self {
            languages: HashMap::new(),
            // rng_cache: rand::thread_rng(),
            rng_cache: rand::rng(),
            result_cache: String::with_capacity(1024),
            words_cache: Vec::with_capacity(100),
        };
        if builder.load_language(DEFAULT_LANGUAGE).is_err() {
            builder.languages.insert(
                DEFAULT_LANGUAGE.to_string(),
                DEFAULT_WORDS.iter().map(|&s| s.to_string()).collect(),
            );
        }
        builder
    }

    /// Test word pool generator.
    pub fn generate_test(&mut self, config: &Config) -> String {
        if let Some(words) = &config.words {
            return words.clone();
        }

        let lang = config.language.as_deref().unwrap_or(DEFAULT_LANGUAGE);

        if !self.languages.contains_key(lang) {
            let _ = self.load_language(lang);
        }

        if !self.languages.contains_key(DEFAULT_LANGUAGE) {
            self.languages.insert(
                DEFAULT_LANGUAGE.to_string(),
                DEFAULT_WORDS.iter().map(|&s| s.to_string()).collect(),
            );
        }

        let base_words = self
            .languages
            .get(lang)
            .or_else(|| self.languages.get(DEFAULT_LANGUAGE))
            .unwrap();

        let word_count = config.resolve_word_count();

        self.words_cache.clear();
        self.words_cache.reserve(word_count);

        if word_count <= base_words.len() {
            let selected_words: Vec<_> = base_words
                .choose_multiple(&mut self.rng_cache, word_count)
                .collect();

            self.words_cache
                .extend(selected_words.iter().map(|&word| word.clone()));
        } else {
            for _ in 0..word_count {
                let idx = self.rng_cache.random_range(0..base_words.len());
                self.words_cache.push(base_words[idx].clone());
            }
        }

        let use_symbols = config.use_symbols;
        let use_punctuation = config.use_punctuation;
        let use_numbers = config.use_numbers;

        let avg_word_len = 5;
        let extra_chars_per_word = 2;
        let total_capacity = word_count * (avg_word_len + extra_chars_per_word);

        self.result_cache.clear();
        if self.result_cache.capacity() < total_capacity {
            self.result_cache
                .reserve(total_capacity - self.result_cache.capacity());
        }

        for (i, word) in self.words_cache.iter().enumerate() {
            log_debug!("index: {i}, word: {word}");
            if i > 0 {
                self.result_cache.push(' ');
            }

            self.result_cache.push_str(word);

            if use_symbols && self.rng_cache.random_bool(SYMBOL_PROBABILITY) {
                let idx = self.rng_cache.random_range(0..SYMBOLS.len());
                self.result_cache.push(SYMBOLS[idx]);
            }
            if use_punctuation && self.rng_cache.random_bool(PUNCTUATION_PROBABILITY) {
                let idx = self.rng_cache.random_range(0..PUNCTUATION.len());
                self.result_cache.push(PUNCTUATION[idx]);
            }
            if use_numbers && self.rng_cache.random_bool(NUMBER_PROBABILITY) {
                let idx = self.rng_cache.random_range(0..NUMBERS.len());
                self.result_cache.push(NUMBERS[idx]);
            }
        }

        self.result_cache.clone()
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

    /// Loads the given language.
    fn load_language(&mut self, lang: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Self::has_language(lang) {
            return Err(format!("Language '{lang}' is not available.").into());
        }

        let content =
            assets::get_language(lang).ok_or_else(|| format!("Language '{lang}' not found"))?;

        let language: Language = serde_json::from_str(&content)?;
        self.languages.insert(language.name, language.words);
        Ok(())
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn print_language_list() {
    let mut languages: Vec<String> = Builder::available_languages().to_vec();
    languages.sort_by_key(|a| a.to_lowercase());

    println!("\n• Available Languages ({}):", languages.len());

    println!("{}", "─".repeat(40));

    for language in languages {
        let is_default = language == DEFAULT_LANGUAGE;
        let language_name = if is_default {
            format!("{language} (default)")
        } else {
            language
        };
        println!("  • {language_name}");
    }
    println!("\nUsage:");
    println!("  • Set language:    termitype --language <name>");
    println!("  • List languages:  termitype --list-languages");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        constants::{MAX_CUSTOM_TIME, WPS_TARGET},
    };

    fn create_builder() -> Builder {
        Builder::new()
    }

    #[test]
    fn test_default_bulder_state() {
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
            .expect("Failed to load spanish language.");
        assert!(builder.languages.contains_key("spanish"));
    }

    #[test]
    fn test_invalid_language() {
        let mut builder = create_builder();
        let result = builder.load_language("invalid_language");
        assert!(result.is_err());
    }

    #[test]
    fn test_word_uniqueness() {
        let mut builder = create_builder();
        let mut config = Config::default();
        config.change_mode(crate::config::ModeType::Words, Some(10));

        let test = builder.generate_test(&config);
        let words: Vec<&str> = test.split_whitespace().collect();

        assert_eq!(words.len(), 10);

        let unique_words: std::collections::HashSet<&str> = words.iter().copied().collect();
        assert!(unique_words.len() > 1);
    }

    #[test]
    fn test_generates_enough_words_in_time_mode() {
        // NOTE: This test assumes a user typing at 300wpm which is equivalent to 5 words per second (300/60)
        let mut builder = create_builder();
        let mut config = Config::default();

        // 10 seconds test duration
        config.change_mode(crate::config::ModeType::Time, Some(10));
        let test = builder.generate_test(&config);
        let words: Vec<&str> = test.split_whitespace().collect();

        assert!(words.len() >= WPS_TARGET as usize * 10);

        // 60 seconds test duration
        config.change_mode(crate::config::ModeType::Time, Some(60));
        let test = builder.generate_test(&config);
        let words: Vec<&str> = test.split_whitespace().collect();
        assert!(words.len() >= WPS_TARGET as usize * 60);

        // 120 seconds test duration
        config.change_mode(crate::config::ModeType::Time, Some(120));
        let test = builder.generate_test(&config);
        let words: Vec<&str> = test.split_whitespace().collect();
        assert!(words.len() >= WPS_TARGET as usize * 120);

        // MAX_CUSTOM_TIME seconds test duration
        config.change_mode(
            crate::config::ModeType::Time,
            Some(MAX_CUSTOM_TIME as usize),
        );
        let test = builder.generate_test(&config);
        let words: Vec<&str> = test.split_whitespace().collect();
        assert!(words.len() >= WPS_TARGET as usize * MAX_CUSTOM_TIME as usize);
    }
}
