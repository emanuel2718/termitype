use std::time::{Duration, Instant};

use crossterm::{
    cursor::SetCursorStyle,
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
    },
};
use ratatui::{prelude::Backend, Terminal};

use crate::{
    actions::{handle_click_action, process_action, TermiAction},
    builder::Builder,
    config::Config,
    input::InputHandler,
    log_debug,
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
    pub preview_ascii_art: Option<String>,
    last_event: Option<KeyEvent>,
    should_quit: bool,
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
            .field("preview_ascii_art", &self.preview_ascii_art)
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
            preview_ascii_art: None,
            last_event: None,
            should_quit: false,
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

        log_debug!("Test started: Generating new words (should trigger UI cache miss)");
        self.words = self.builder.generate_test(&self.config);
        self.tracker = Tracker::new(&self.config, self.words.clone());

        self.menu = menu;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Redo Redo - Taeha circa 2020
    pub fn redo(&mut self) {
        if self.handle_debounce() {
            return;
        }

        let menu = self.menu.clone();
        let words = self.words.clone();

        log_debug!("Test redo: Resetting tracker (UI cache should remain valid if same words)");
        self.tracker = Tracker::new(&self.config, words);
        self.menu = menu;
    }

    pub fn current_theme(&self) -> &Theme {
        self.preview_theme.as_ref().unwrap_or(&self.theme)
    }

    fn handle_debounce(&self) -> bool {
        if self.tracker.status == Status::Completed {
            if let Some(event) = self.last_event {
                if event.code == KeyCode::Enter {
                    return false;
                }
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

    let typing_frame_time = Duration::from_micros(2778); // ~360 FPS for extreme responsiveness
    let idle_frame_time = Duration::from_millis(33); // ~30 FPS when idle (energy efficient)

    // FPS tracking
    let mut last_keystroke = Instant::now();
    let fps_update_interval = Duration::from_millis(500);

    let mut current_fps: f64 = 0.0;
    let mut last_fps_update_time = Instant::now();
    let mut frame_count = 0u32;
    let mut last_frame_time = Instant::now();

    let mut needs_redraw = true;

    'main_loop: loop {
        if termi.should_quit {
            break 'main_loop;
        }
        let now = Instant::now();

        // update fps if we are showing them
        if termi.config.show_fps {
            frame_count += 1;
            if now.duration_since(last_fps_update_time) >= fps_update_interval {
                let elapsed = now.duration_since(last_fps_update_time).as_secs_f64();
                current_fps = frame_count as f64 / elapsed;
                frame_count = 0;
                last_fps_update_time = now;
                needs_redraw = true;
            }
        }

        // adaptive frame rate
        let is_actively_typing = now.duration_since(last_keystroke) < Duration::from_millis(1500);
        let is_active_state =
            termi.tracker.status == Status::Typing || termi.menu.is_open() || termi.modal.is_some();

        let target_frame_time = if is_actively_typing || is_active_state {
            typing_frame_time
        } else {
            idle_frame_time
        };

        // frame rate limit to avoid wutututututu
        let frame_elapsed = now.duration_since(last_frame_time);
        if frame_elapsed < target_frame_time {
            let sleep_time = target_frame_time - frame_elapsed;
            std::thread::sleep(sleep_time);
        }
        last_frame_time = Instant::now();

        let timeout = Duration::from_millis(1);
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(event) => {
                    if event.kind == KeyEventKind::Press {
                        let last_event = termi.last_event;
                        termi.last_event = Some(event);
                        let input_mode = input_handler.resolve_input_mode(&termi);
                        let action = input_handler.handle_input(event, last_event, input_mode);

                        // inmediate actions
                        if action == TermiAction::Quit {
                            break 'main_loop;
                        }

                        last_keystroke = Instant::now();

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

        // `Time` mode time tracking shennaningans
        if termi.tracker.status == Status::Typing {
            if let Some(end_time) = termi.tracker.time_end {
                let new_time_remaining = if Instant::now() >= end_time {
                    Duration::from_secs(0)
                } else {
                    end_time.duration_since(Instant::now())
                };

                // only update `time_remaining` if seconds have changed
                let current_seconds = termi
                    .tracker
                    .time_remaining
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let new_seconds = new_time_remaining.as_secs();

                if current_seconds != new_seconds {
                    termi.tracker.time_remaining = Some(new_time_remaining);
                    needs_redraw = true;
                }
            }
        }

        // lazyly update metrics
        if termi.tracker.needs_metrics_update() || termi.tracker.status == Status::Typing {
            termi.tracker.update_metrics();
            if termi.tracker.status == Status::Typing {
                needs_redraw = true;
            }
        }

        let fps_to_display = if termi.config.show_fps {
            Some(current_fps)
        } else {
            None
        };

        // render if we need to render
        if needs_redraw {
            terminal.draw(|frame| {
                click_regions = draw_ui(frame, &mut termi, fps_to_display);
            })?;
            needs_redraw = false;
        }
    }

    Ok(())
}
