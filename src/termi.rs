use std::{
    error::Error,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event};
use ratatui::{prelude::Backend, Terminal};

use crate::{generator::Generator, input::handle_input, ui::draw_ui, version::VERSION};

pub struct Termi {
    pub title: String,
    pub user_input: Vec<Option<char>>,
    pub cursor_pos: usize,
    pub target_text: String,
    pub is_finished: bool,
    pub start_time: Instant,
    pub duration: Duration,
    pub time_remaining: Duration,
}

impl Termi {
    pub fn new() -> Self {
        // TODO: make generator.rs to generate the target_text
        let generator = Generator::new("assets/10k.txt").expect("Failed to load the word list");
        let target_text = generator.generate(10);
        // the title will be TermiType plus the VERSION
        Termi {
            title: format!("TermiType {}", VERSION),
            user_input: vec![None; target_text.chars().count()],
            target_text,
            cursor_pos: 0,
            is_finished: false,
            start_time: Instant::now(),
            duration: Duration::from_secs(60), // TODO: get this from args
            time_remaining: Duration::from_secs(60),
        }
    }

    fn on_tick(&mut self) {
        if !self.is_finished {
            let elapsed = self.start_time.elapsed();
            if elapsed >= self.duration {
                self.is_finished = true;
                self.time_remaining = Duration::from_secs(0);
            } else {
                self.time_remaining = self.duration - elapsed;
            }
        }
    }

    pub fn check_completion(&mut self) {
        if self.cursor_pos >= self.target_text.chars().count() {
            self.is_finished = true;
        }
    }

    pub fn restart(&mut self) {
        self.user_input = vec![None; self.target_text.chars().count()];
        self.start_time = Instant::now();
        self.is_finished = false;
        self.cursor_pos = 0;
    }
}

pub fn run_termi<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    let mut termi = Termi::new();
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw_ui(f, &termi))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if handle_input(key, &mut termi) {
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
