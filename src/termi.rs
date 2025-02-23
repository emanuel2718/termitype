use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event, MouseButton, MouseEvent, MouseEventKind},
    execute,
};
use ratatui::{prelude::Backend, Terminal};

use crate::{
    builder::Builder,
    config::Config,
    input::{process_action, Action, InputHandler},
    menu::{MenuAction, MenuState},
    theme::Theme,
    tracker::Tracker,
    ui::{
        components::{ClickAction, ClickableRegion},
        draw_ui,
    },
};

#[cfg(debug_assertions)]
use crate::debug::{Debug, LOG};

pub struct Termi {
    pub config: Config,
    pub tracker: Tracker,
    pub theme: Theme,
    pub preview_theme: Option<Theme>,
    pub preview_cursor: Option<SetCursorStyle>,
    pub builder: Builder,
    pub words: String,
    pub menu: MenuState,
    pub clickable_regions: Vec<ClickableRegion>,
    #[cfg(debug_assertions)]
    pub debug: Option<Debug>,
}

impl std::fmt::Debug for Termi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Termi");
        debug_struct
            .field("config", &self.config)
            .field("tracker", &self.tracker)
            .field("theme", &self.theme)
            .field("preview_theme", &self.preview_theme)
            .field(
                "preview_cursor",
                &self
                    .config
                    .resolve_cursor_name_from_style(&self.preview_cursor),
            )
            .field("builder", &self.builder)
            .field("words", &self.words)
            .field("menu", &self.menu)
            .field("clickable_regions", &self.clickable_regions);

        #[cfg(debug_assertions)]
        debug_struct.field("debug", &self.debug);

        debug_struct.finish()
    }
}

impl Termi {
    pub fn new(config: &Config) -> Self {
        let theme = Theme::new(config);
        let mut builder = Builder::new();
        let words = builder.generate_test(config);
        let tracker = Tracker::new(config, words.clone());
        let menu = MenuState::new();

        #[cfg(debug_assertions)]
        let debug = if config.debug {
            Some(Debug::new())
        } else {
            None
        };

        Self {
            config: config.clone(),
            tracker,
            theme,
            preview_theme: None,
            preview_cursor: None,
            builder,
            words,
            menu,
            clickable_regions: Vec::new(),
            #[cfg(debug_assertions)]
            debug,
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
                    ClickAction::TogglePunctuation => {
                        self.config.toggle_punctuation();
                        self.start();
                    }
                    ClickAction::ToggleNumbers => {
                        self.config.toggle_numbers();
                        self.start();
                    }
                    ClickAction::ToggleSymbols => {
                        self.config.toggle_symbols();
                        self.start();
                    }
                    ClickAction::SwitchMode(mode) => {
                        self.config.change_mode(mode, None);
                        self.start();
                    }
                    ClickAction::SetModeValue(value) => {
                        self.config.change_mode_value(value);
                        self.start();
                    }
                    ClickAction::OpenThemePicker => {
                        self.menu.toggle(&self.config);
                        self.menu.execute(MenuAction::OpenThemePicker, &self.config);
                        self.menu.preview_selected();
                        self.update_preview_theme();
                    }
                    ClickAction::OpenLanguagePicker => {
                        self.menu.toggle(&self.config);
                        self.menu
                            .execute(MenuAction::OpenLanguagePicker, &self.config);
                        self.menu.preview_selected();
                    }
                }
                break;
            }
        }
    }

    pub fn start(&mut self) {
        let menu = self.menu.clone();
        let preview_theme = self.preview_theme.clone();
        let preview_cursor = self.preview_cursor;
        #[cfg(debug_assertions)]
        let debug = self.debug.clone();

        // hmm, if the use passed the words flag, should we reset it?
        // maybe the user wants to pracitice those sepecific words over and over again?
        // reset words flag for now...
        if self.config.words.is_some() {
            self.config.reset_words_flag();
        }

        *self = Termi::new(&self.config);
        self.menu = menu;
        self.preview_theme = preview_theme;
        self.preview_cursor = preview_cursor;
        #[cfg(debug_assertions)]
        {
            self.debug = debug;
        }
    }

    pub fn get_current_theme(&self) -> &Theme {
        self.preview_theme.as_ref().unwrap_or(&self.theme)
    }

    pub fn update_preview_theme(&mut self) {
        if let Some(theme_name) = self.menu.get_preview_theme() {
            self.preview_theme = Some(Theme::from_name(theme_name));
        } else {
            self.preview_theme = None;
        }
    }

    pub fn update_preview_cursor(&mut self) {
        if let Some(cursor_name) = self.menu.get_preview_cursor() {
            let cursor_style = self.config.resolve_cursor_style_from_name(cursor_name);
            self.preview_cursor = Some(cursor_style);
            execute!(std::io::stdout(), cursor_style).ok();
        } else {
            self.preview_cursor = None;
            execute!(
                std::io::stdout(),
                self.config.resolve_current_cursor_style()
            )
            .ok();
        }
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> Result<()> {
    let mut termi = Termi::new(config);
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
                    #[cfg(debug_assertions)]
                    let action = input_handler.handle_input(key, &termi.menu, termi.config.debug);
                    #[cfg(not(debug_assertions))]
                    let action = input_handler.handle_input(key, &termi.menu, false);

                    if action == Action::Quit {
                        break;
                    }

                    #[cfg(debug_assertions)]
                    if termi.debug.is_some() {
                        LOG(format!(
                            "Key Event - code: {:?}, modifiers: {:?}, action: {:?}, menu_open: {}",
                            key.code,
                            key.modifiers,
                            action,
                            termi.menu.is_open()
                        ));
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
            #[cfg(debug_assertions)]
            if let Some(debug) = termi.debug.as_mut() {
                debug.sync_with_global();
            }
            last_tick = Instant::now()
        }
    }

    Ok(())
}
