use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use rand::{seq::SliceRandom, thread_rng};

pub struct Generator {
    pub words: Vec<String>,
}

impl Generator {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut words = Vec::new();

        for line in reader.lines() {
            let word = line?;
            if !word.is_empty() {
                words.push(word)
            }
        }
        words.shrink_to_fit();
        Ok(Self { words })
    }

    pub fn generate(&self, word_count: usize) -> String {
        let mut rng = thread_rng();

        if self.words.is_empty() {
            return String::new();
        }

        if word_count <= self.words.len() {
            let sample = self
                .words
                .choose_multiple(&mut rng, word_count)
                .cloned()
                .collect::<Vec<String>>();
            sample.join(" ")
        } else {
            let sample = (0..word_count)
                .map(|_| self.words.choose(&mut rng).unwrap().clone())
                .collect::<Vec<String>>();
            sample.join(" ")
        }
    }
}
