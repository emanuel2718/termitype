use std::{collections::HashMap, fs, path::PathBuf, sync::OnceLock};

use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{config::Config, constants::DEFAULT_LANGUAGE};

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
}

impl Builder {
    /// Creates a new Builder instance with the give language.
    pub fn new() -> Self {
        let mut builder = Self {
            languages: HashMap::new(),
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
        let lang = config.language.as_deref().unwrap_or(DEFAULT_LANGUAGE);

        if config.words.is_some() {
            return config.words.clone().unwrap();
        }

        // load given language ahead of time
        if !self.languages.contains_key(lang) {
            let _ = self.load_language(lang);
        }

        let base_words = self
            .languages
            .get(lang)
            .or_else(|| self.languages.get(DEFAULT_LANGUAGE))
            .unwrap_or_else(|| &self.languages[DEFAULT_LANGUAGE]);

        let mut rng = thread_rng();
        let word_count = config.resolve_word_count();

        let mut final_words: Vec<String> = if word_count <= base_words.len() {
            base_words
                .choose_multiple(&mut rng, word_count)
                .cloned()
                .collect()
        } else {
            (0..word_count)
                .map(|_| base_words.choose(&mut rng).unwrap().clone())
                .collect()
        };

        // modifiers
        for word in &mut final_words {
            if config.use_symbols && rng.gen_bool(SYMBOL_PROBABILITY) {
                word.push(*SYMBOLS.choose(&mut rng).unwrap());
            }
            if config.use_punctuation && rng.gen_bool(PUNCTUATION_PROBABILITY) {
                word.push(*PUNCTUATION.choose(&mut rng).unwrap());
            }
            if config.use_numbers && rng.gen_bool(NUMBER_PROBABILITY) {
                word.push(*NUMBERS.choose(&mut rng).unwrap());
            }
        }

        final_words.join(" ")
    }

    /// Returns the list of available languages.
    pub fn available_languages() -> &'static [String] {
        static LANGUAGES: OnceLock<Vec<String>> = OnceLock::new();
        LANGUAGES.get_or_init(|| {
            Self::load_list_of_available_languages()
                .unwrap_or_else(|_| vec![DEFAULT_LANGUAGE.to_string()])
        })
    }

    /// Checks if the given language is available.
    pub fn has_language(language: &str) -> bool {
        Self::available_languages().contains(&language.to_string())
    }

    /// Loads the given language.
    fn load_language(&mut self, lang: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Self::has_language(lang) {
            return Err(format!("Language '{}' is not available.", lang).into());
        }
        let path = PathBuf::from(format!("assets/languages/{lang}.json"));
        let content = fs::read_to_string(path)?;
        let language: Language = serde_json::from_str(&content)?;

        self.languages.insert(language.name, language.words);
        Ok(())
    }

    /// Loads the list of available languages.
    fn load_list_of_available_languages() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let path = PathBuf::from("assets/languages/_list.json");
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn create_builder() -> Builder {
        let builder = Builder::new();
        builder
    }

    #[test]
    fn test_default_bulder_state() {
        let builder = create_builder();
        assert_eq!(builder.languages.is_empty(), false);
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

        // check we got the requested number of words
        assert_eq!(words.len(), 10);

        // check that words are not all the same
        let unique_words: std::collections::HashSet<&str> = words.iter().copied().collect();
        assert!(unique_words.len() > 1);
    }
}
