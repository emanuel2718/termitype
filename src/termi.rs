use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{prelude::Backend, Terminal};

use crate::{
    builder::Builder,
    config::Config,
    input::{process_action, Action, InputHandler},
    menu::Menu,
    theme::Theme,
    tracker::Tracker,
    ui::{
        components::{ClickAction, ClickableRegion},
        draw_ui,
    },
};

#[derive(Debug)]
pub struct Termi {
    pub config: Config,
    pub tracker: Tracker,
    pub theme: Theme,
    pub builder: Builder,
    pub words: String,
    pub menu: Menu,
    pub clickable_regions: Vec<ClickableRegion>,
}

impl Termi {
    pub fn new(config: &Config) -> Self {
        let theme = Theme::new(config);
        let mut builder = Builder::new();
        let words = builder.generate_test(config);
        let tracker = Tracker::new(config, words.clone());
        Termi {
            config: config.clone(),
            tracker,
            theme,
            builder,
            words,
            menu: Menu::default(),
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
        let menu = self.menu.clone();
        *self = Termi::new(&self.config);
        // TODO: eventually we would want to restore previous state. (themes come to mind)
        self.menu = menu; // restore menu state
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> Result<()> {
    let mut termi = Termi::new(config);

    let tick_rate = Duration::from_millis(500);
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
                    let action = input_handler.handle_input(key, termi.menu.visible);
                    if action == Action::Quit {
                        break;
                    }
                    let action = process_action(action, &mut termi);
                    if action == Action::Quit {
                        break;
                    }
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
