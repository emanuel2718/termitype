use std::{
    error::Error,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event};
use ratatui::{prelude::Backend, Terminal};

use crate::{
    config::{Config, Mode}, generator::Generator, input::InputHandler, ui::draw_ui, version::VERSION
};

pub struct Termi {
    pub title: String,
    pub user_input: Vec<Option<char>>,
    pub cursor_pos: usize,
    pub target_text: String,
    pub is_finished: bool,
    pub is_started: bool,
    pub start_time: Instant,

    pub duration: Duration,
    pub time_remaining: Duration, // for time based
    pub total_words: usize,       // for word mode
    pub wpm: f64,
    pub correct_chars: usize,
    pub mode: Mode,
}

// TODO: get this from cli args
static WORD_FILE: &str = "assets/100.txt";

impl Termi {
    pub fn new(config: &Config) -> Self {
        let generator = Generator::new(WORD_FILE).expect("Failed to load the word list");

        let target_text = match config.mode {
            Mode::Time => generator.generate(100),
            Mode::Words => generator.generate(config.words),
        };

        let duration = match config.mode {
            Mode::Time => config.time,
            Mode::Words => 0,
        };

        Termi {
            title: format!("TermiType {}", VERSION),
            user_input: vec![None; target_text.chars().count()],
            target_text,
            duration: Duration::from_secs(duration),
            time_remaining: Duration::from_secs(duration),
            mode: config.mode,
            total_words: config.words,
            cursor_pos: 0,
            is_finished: false,
            is_started: false,
            start_time: Instant::now(),
            wpm: 0.0,
            correct_chars: 0,
        }
    }

    pub fn check_completion(&mut self) {
        if self.cursor_pos >= self.target_text.chars().count() {
            self.is_finished = true;
        }
    }

    pub fn reset(&mut self) {
        let generator = Generator::new(WORD_FILE).expect("Failed to load words");
        self.target_text = match self.mode {
            Mode::Time => generator.generate(100),
            Mode::Words => generator.generate(self.total_words),
        };

        self.user_input = vec![None; self.target_text.chars().count()];
        self.is_finished = false;
        self.is_started = false;
        self.cursor_pos = 0;
        self.correct_chars = 0;
        self.wpm = 0.0;
        self.duration = self.duration;
        self.time_remaining = self.duration;
    }

    pub fn start(&mut self) {
        self.start_time = Instant::now();
        self.is_started = true;
    }

    fn on_tick(&mut self) {
        match self.mode {
            Mode::Time => {
                if !self.is_finished && self.is_started {
                    let elapsed = self.start_time.elapsed();
                    if elapsed >= self.duration {
                        self.is_finished = true;
                        self.time_remaining = Duration::from_secs(0);
                    } else {
                        self.time_remaining = self.duration - elapsed;
                    }
                    self.update_wpm();
                }
            }
            Mode::Words => {
                if !self.is_finished && self.is_started {
                    self.update_wpm();
                }
            }
        }
    }

   pub fn update_wpm(&mut self) {
        if self.is_started {
            let elapsed_minutes = self.start_time.elapsed().as_secs_f64() / 60.0;
            let correct_words_typed = self.correct_chars as f64 / 5.0;
            self.wpm = correct_words_typed / elapsed_minutes;
        } else {
            self.wpm = 0.0;
        }
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> Result<(), Box<dyn Error>> {
    let mut termi = Termi::new(&config);
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    let mut input_handler = InputHandler::new();

    loop {
        terminal.draw(|f| draw_ui(f, &termi))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if input_handler.handle_input(key, &mut termi) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if !termi.is_finished {
                termi.on_tick()
            }
            last_tick = Instant::now();
        }
    }

    Ok(())
}
