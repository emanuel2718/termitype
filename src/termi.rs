use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{prelude::Backend, Terminal};

use crate::{
    builder::Builder,
    config::Config,
    input::{process_action, Action, InputHandler},
    theme::Theme,
    tracker::Tracker, ui::{components::{ClickAction, ClickableRegion}, draw_ui},
};

#[derive(Debug)]
pub struct Termi {
    pub config: Config,
    pub tracker: Tracker,
    pub theme: Theme,
    pub builder: Builder,
    pub words: String,
    pub clickable_regions: Vec<ClickableRegion>,
}

impl Termi {
    pub fn new(config: &Config) -> Self {
        let theme = Theme::new(&config);
        let mut builder = Builder::new();
        let words = builder.generate_test(config);
        let tracker = Tracker::new(&config, words.clone());
        Termi {
            config: config.clone(),
            tracker,
            theme,
            builder,
            words,
            clickable_regions: Vec::new(),
        }
    }
    pub fn handle_click(&mut self, x: u16, y: u16) {
        for region in &self.clickable_regions {
            if x >= region.area.x 
                && x < region.area.x + region.area.width
                && y >= region.area.y 
                && y < region.area.y + region.area.height 
            {
                match region.action {
                    ClickAction::TogglePunctuation => self.config.toggle_punctuation(),
                    ClickAction::ToggleNumbers => self.config.toggle_numbers(),
                    ClickAction::SwitchMode(mode) => self.config.change_mode(mode, None),
                    ClickAction::SetModeValue(value) => self.config.change_mode_value(value),
                }
                self.start();
                break;
            }
        }
    }

    pub fn start(&mut self) {
        *self = Termi::new(&self.config);
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> Result<()> {
    let mut termi = Termi::new(&config);

    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    let mut input_handler = InputHandler::new();

    loop {
        terminal.draw(|f| draw_ui(f, &mut termi))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    let action = input_handler.handle_input(key);
                    if action == Action::Quit {
                        break;
                    }
                    process_action(action, &mut termi);
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    ..
                }) => {
                    termi.handle_click(column, row);
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            termi.tracker.update_metrics();
            last_tick = Instant::now()
        }
    }

    Ok(())
}
