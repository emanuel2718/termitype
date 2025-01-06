use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

use rand::{seq::SliceRandom, thread_rng};

use crate::{config::Config, constants::DEFAULT_LANGUAGE};

fn get_language_path(language: &str) -> PathBuf {
    let lang_path = PathBuf::from(format!("assets/{language}"));
    if lang_path.is_file() {
        lang_path
    } else {
        PathBuf::from(format!("assets/{DEFAULT_LANGUAGE}"))
    }
}

fn read_from_file(path: &PathBuf) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let words: Vec<String> = reader
        .lines()
        .filter_map(|line| match line {
            Ok(word) if !word.is_empty() => Some(word),
            _ => None,
        })
        .collect();

    Ok(words)
}

pub fn generate_test(config: &Config) -> String {
    let lang = config
        .language
        .clone()
        .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string());

    let lang_path = get_language_path(&lang);
    let words = match read_from_file(&lang_path) {
        Ok(words) => words,
        Err(_) => {
            let fallback_path = get_language_path(DEFAULT_LANGUAGE);
            read_from_file(&fallback_path).unwrap_or_default()
        }
    };

    let mut rng = thread_rng();
    let word_count = config.resolve_word_count();

    if words.is_empty() {
        return String::from("[NO WORDS AVAILABLE]");
    }

    if word_count <= words.len() {
        words
            .choose_multiple(&mut rng, word_count)
            .cloned()
            .collect::<Vec<String>>()
            .join(" ")
    } else {
        (0..word_count)
            .map(|_| words.choose(&mut rng).unwrap().clone())
            .collect::<Vec<String>>()
            .join(" ")
    }
}
