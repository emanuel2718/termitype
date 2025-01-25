use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::{prelude::Backend, Terminal};

use crate::{
    builder::Builder, config::Config, input::InputHandler, renderer::draw_ui, theme::Theme,
    tracker::Tracker,
};

#[derive(Debug)]
pub struct Termi {
    pub config: Config,
    pub tracker: Tracker,
    pub theme: Theme,
    pub builder: Builder,
    pub words: String,
}

impl Termi {
    pub fn new(config: &Config) -> Self {
        let tracker = Tracker::new(&config);
        let theme = Theme::new(&config);
        let mut builder = Builder::new();
        let words = builder.generate_test(config);
        Termi {
            config: config.clone(),
            tracker,
            theme,
            builder,
            words,
        }
    }

    pub fn start(&mut self) {
        *self = Termi::new(&self.config)
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> Result<()> {
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
            last_tick = Instant::now()
        }
    }

    Ok(())
}
