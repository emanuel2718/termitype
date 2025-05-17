use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind},
};
use ratatui::{prelude::Backend, Terminal};

use crate::{
    actions::{handle_click_action, process_action, TermiAction},
    builder::Builder,
    config::Config,
    input::InputHandler,
    menu::MenuState,
    modal::InputModal,
    theme::Theme,
    tracker::{Status, Tracker},
    ui::{draw_ui, render::TermiClickableRegions},
};

pub struct Termi {
    pub config: Config,
    pub tracker: Tracker,
    pub theme: Theme,
    pub builder: Builder,
    pub words: String,
    pub menu: MenuState,
    pub modal: Option<InputModal>,
    pub preview_theme: Option<Theme>,
    pub preview_cursor: Option<SetCursorStyle>,
    pub last_key: Option<KeyCode>,
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
            .field("menu", &self.menu)
            .field("modal", &self.modal);
        debug_struct.finish()
    }
}

impl Termi {
    pub fn new(config: &Config) -> Self {
        let mut builder = Builder::new();
        let words = builder.generate_test(config);

        Self {
            config: config.clone(),
            tracker: Tracker::new(config, words.clone()),
            theme: Theme::new(config),
            menu: MenuState::new(),
            builder,
            words,
            modal: None,
            preview_theme: None,
            preview_cursor: None,
            last_key: None,
        }
    }

    pub fn start(&mut self) {
        if self.handle_debounce() {
            return;
        }

        let menu = self.menu.clone();
        if self.config.words.is_some() {
            self.config.reset_words_flag();
        }

        self.words = self.builder.generate_test(&self.config);
        self.tracker = Tracker::new(&self.config, self.words.clone());

        self.menu = menu;
    }

    /// Redo Redo - Taeha circa 2020
    pub fn redo(&mut self) {
        if self.handle_debounce() {
            return;
        }

        let menu = self.menu.clone();
        let words = self.words.clone();

        self.tracker = Tracker::new(&self.config, words);
        self.menu = menu;
    }

    pub fn current_theme(&self) -> &Theme {
        self.preview_theme.as_ref().unwrap_or(&self.theme)
    }

    fn handle_debounce(&self) -> bool {
        if self.tracker.status == Status::Completed {
            // NOTE(ema): handling this here is extremely lazy on my part. This should be handled by
            //  the input handler
            if self.last_key == Some(KeyCode::Enter) {
                return false;
            }

            if let Some(end_time) = self.tracker.time_end {
                if end_time.elapsed() < Duration::from_millis(500) {
                    return true;
                }
            }
        }
        false
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> anyhow::Result<()> {
    let mut termi = Termi::new(config);
    let mut input_handler = InputHandler::new();
    let mut click_regions = TermiClickableRegions::default();

    let mut frame_times: VecDeque<Instant> = VecDeque::with_capacity(60);
    let typing_frame_time = Duration::from_micros(6944); // ~144 FPS (1000000/144)
    let idle_frame_time = Duration::from_millis(100); // slower refresh when IDLE

    let mut last_tick = Instant::now();
    let mut last_metrics_update = Instant::now();
    let mut last_keystroke = Instant::now();

    let mut current_fps: f64 = 0.0;
    let mut last_fps_update_time = Instant::now();
    let fps_update_interval = Duration::from_millis(500);

    let mut needs_redraw = true;

    'main_loop: loop {
        let now = Instant::now();

        let current_frame_time =
            if now.duration_since(last_keystroke) < Duration::from_secs(1) || config.show_fps {
                typing_frame_time
            } else {
                idle_frame_time
            };

        let timeout = current_frame_time
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(event) => {
                    if event.kind == KeyEventKind::Press {
                        termi.last_key = Some(event.code);
                        let input_mode = input_handler.resolve_input_mode(&termi);
                        let action = input_handler.handle_input(event, input_mode);

                        // inmediate actions
                        if action == TermiAction::Quit {
                            break 'main_loop;
                        }

                        last_keystroke = now;

                        // process all the actions that are not quit as is the only inmediate action atm
                        process_action(action, &mut termi);
                        needs_redraw = true;
                    }
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    ..
                }) => {
                    let click_action = handle_click_action(&mut termi, &click_regions, column, row);
                    if let Some(action) = click_action {
                        process_action(action, &mut termi);
                        needs_redraw = true;
                    }
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
                click_regions = draw_ui(frame, &mut termi, fps_to_display);
            })?;
            needs_redraw = false;
        }

        last_tick = Instant::now();
    }

    Ok(())
}
