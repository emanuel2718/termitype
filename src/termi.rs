use std::collections::VecDeque;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event, KeyEventKind, MouseButton, MouseEvent, MouseEventKind},
    execute,
};
use ratatui::{layout::Position, prelude::Backend, Terminal};

use crate::config::ModeType;
use crate::modal::{build_modal, InputModal, ModalContext};
use crate::{
    builder::Builder,
    config::Config,
    input::{process_action, Action, InputHandler},
    log_debug,
    menu::{MenuAction, MenuState},
    theme::Theme,
    tracker::Tracker,
    ui::{actions::TermiClickAction, draw_ui, render::TermiClickableRegions},
};

pub struct Termi {
    pub config: Config,
    pub tracker: Tracker,
    pub theme: Theme,
    pub preview_theme: Option<Theme>,
    pub preview_theme_name: Option<String>,
    pub preview_cursor: Option<SetCursorStyle>,
    pub builder: Builder,
    pub words: String,
    pub menu: MenuState,
    pub modal: Option<InputModal>,
}

impl std::fmt::Debug for Termi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Termi");
        debug_struct
            .field("config", &self.config)
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
            .field("menu", &self.words)
            .field("modal", &self.modal);

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

        Self {
            config: config.clone(),
            tracker,
            theme,
            preview_theme: None,
            preview_theme_name: None,
            preview_cursor: None,
            builder,
            words,
            menu,
            modal: None,
        }
    }

    pub fn handle_click(&mut self, reg: &TermiClickableRegions, x: u16, y: u16) {
        let mut clicked_action: Option<TermiClickAction> = None;
        for (rect, action) in reg.regions.iter().rev() {
            if rect.contains(Position { x, y }) {
                clicked_action = Some(*action);
                break;
            }
        }

        if let Some(action) = clicked_action {
            match action {
                TermiClickAction::SwitchMode(mode) => {
                    self.config.change_mode(mode, None);
                    self.start();
                }
                TermiClickAction::SetModeValue(value) => {
                    self.config.change_mode_value(value);
                    self.start();
                }
                TermiClickAction::TogglePunctuation => {
                    self.config.toggle_punctuation();
                    self.start();
                }
                TermiClickAction::ToggleSymbols => {
                    self.config.toggle_symbols();
                    self.start();
                }
                TermiClickAction::ToggleNumbers => {
                    self.config.toggle_numbers();
                    self.start();
                }
                TermiClickAction::ToggleThemePicker => {
                    if self.theme.color_support.supports_themes() {
                        if self.menu.get_preview_theme().is_some() {
                            self.menu.close();
                            self.update_preview_theme();
                        } else {
                            self.menu
                                .toggle_from_footer(&self.config, MenuAction::ToggleThemePicker);
                            self.update_preview_theme();
                            self.menu.preview_selected();
                        }
                    }
                }
                TermiClickAction::ToggleLanguagePicker => {
                    self.menu
                        .toggle_from_footer(&self.config, MenuAction::OpenLanguagePicker);
                    self.menu.preview_selected();
                }
                TermiClickAction::ToggleAbout => {
                    self.menu
                        .toggle_from_footer(&self.config, MenuAction::OpenAbout);
                }
                TermiClickAction::ToggleMenu => {
                    self.menu.toggle_menu(&self.config);
                }
                TermiClickAction::ToggleModal(ctx) => {
                    if self.modal.is_some() {
                        self.modal = None;
                    } else {
                        self.modal = Some(build_modal(ctx));
                    }
                }
                TermiClickAction::ModalConfirm => {
                    // NOTE: this is repeated on input.rs
                    if let Some(modal) = self.modal.as_mut() {
                        if modal.buffer.error_msg.is_some() || modal.buffer.input.is_empty() {
                            return;
                        }
                        match modal.ctx {
                            ModalContext::CustomTime => {
                                let value = modal.get_value().parse::<usize>();
                                if value.is_err() {
                                    return;
                                }
                                self.config
                                    .change_mode(ModeType::Time, Some(value.unwrap()));
                                self.start();
                            }
                            ModalContext::CustomWordCount => {
                                let value = modal.get_value().parse::<usize>();
                                if value.is_err() {
                                    return;
                                }
                                self.config
                                    .change_mode(ModeType::Words, Some(value.unwrap()));
                                self.start();
                            }
                        }
                    }
                    self.modal = None;
                }
            }
        }
    }

    pub fn start(&mut self) {
        let menu = self.menu.clone();
        let preview_theme = self.preview_theme.clone();
        let preview_theme_name = self.preview_theme_name.clone();
        let preview_cursor = self.preview_cursor;

        if self.config.words.is_some() {
            self.config.reset_words_flag();
        }

        self.words = self.builder.generate_test(&self.config);

        self.tracker = Tracker::new(&self.config, self.words.clone());

        self.menu = menu;
        self.preview_theme = preview_theme;
        self.preview_theme_name = preview_theme_name;
        self.preview_cursor = preview_cursor;
    }

    pub fn get_current_theme(&self) -> &Theme {
        self.preview_theme.as_ref().unwrap_or(&self.theme)
    }

    pub fn clear_previews(&mut self) {
        self.preview_theme = None;
        self.preview_cursor = None
    }

    pub fn update_preview_theme(&mut self) {
        if let Some(theme_name) = self.menu.get_preview_theme() {
            self.preview_theme_name = Some(theme_name.clone());

            let needs_update = match &self.preview_theme {
                None => true,
                Some(current_theme) => current_theme.id != *theme_name,
            };

            if needs_update {
                self.preview_theme = Some(Theme::from_name(theme_name));
            }
        } else {
            self.preview_theme_name = None;
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

    let frame_time = Duration::from_micros(6944); // ~144 FPS (1000000/144)
    let idle_frame_time = Duration::from_millis(100); // slower refresh when IDLE

    let mut last_tick = Instant::now();
    let mut last_metrics_update = Instant::now();
    let mut last_keystroke = Instant::now();
    let mut input_handler = InputHandler::new();
    let mut needs_redraw = true;
    let mut clickable_regions = TermiClickableRegions::default();
    let mut frame_times: VecDeque<Instant> = VecDeque::with_capacity(60);
    let mut current_fps: f64 = 0.0;
    let mut last_fps_update_time = Instant::now();
    let fps_update_interval = Duration::from_millis(500);

    loop {
        let now = Instant::now();

        let current_frame_time =
            if now.duration_since(last_keystroke) < Duration::from_secs(1) || config.show_fps {
                frame_time
            } else {
                idle_frame_time
            };

        let timeout = current_frame_time
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        let is_modal_active = termi.modal.is_some();
                        let action = input_handler.handle_input(key, &termi.menu, is_modal_active);

                        if action == Action::Quit {
                            break;
                        }

                        last_keystroke = now;
                        let action = process_action(action, &mut termi);
                        needs_redraw = true;

                        if action == Action::Quit {
                            break;
                        }
                    }
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    ..
                }) => {
                    log_debug!("Status: {:?}", termi.tracker.status);
                    termi.handle_click(&clickable_regions, column, row);
                    needs_redraw = true;
                }
                Event::Resize(_width, _height) => {
                    needs_redraw = true;
                }
                _ => {}
            }
        }

        if termi.tracker.should_complete() {
            termi.tracker.complete();
            needs_redraw = true;
        }

        if now.duration_since(last_metrics_update) >= Duration::from_millis(500) {
            termi.tracker.update_metrics();
            last_metrics_update = now;
        }

        frame_times.push_back(now);
        if frame_times.len() > 60 {
            frame_times.pop_front();
        }

        if frame_times.len() > 1 {
            if let (Some(newest), Some(oldest)) = (frame_times.back(), frame_times.front()) {
                let duration = newest.duration_since(*oldest);
                if duration > Duration::ZERO {
                    let avg_frame_time = duration.as_secs_f64() / (frame_times.len() - 1) as f64;
                    if avg_frame_time > 0.0 {
                        current_fps = 1.0 / avg_frame_time;
                    }
                }
            }
        }

        let fps_to_display = if config.show_fps {
            Some(current_fps)
        } else {
            None
        };

        if config.show_fps
            && !needs_redraw
            && now.duration_since(last_fps_update_time) >= fps_update_interval
        {
            needs_redraw = true;
            last_fps_update_time = now;
        }

        if needs_redraw {
            terminal.draw(|frame| {
                clickable_regions = draw_ui(frame, &mut termi, fps_to_display);
            })?;
            needs_redraw = false;
        }

        last_tick = Instant::now();
    }

    Ok(())
}
