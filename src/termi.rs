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
    ui::{components::elements::TermiClickableRegions, draw_ui},
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

    const TYPING_FRAME_TIME: Duration = Duration::from_millis(4); // ~240 FPS
    const IDLE_FRAME_TIME: Duration = Duration::from_millis(300); // ~30 FPS when idle

    let mut last_input_at = Instant::now();
    let mut last_tick = Instant::now();
    let mut last_metrics_update = Instant::now();
    let mut last_time_update = Instant::now();
    let mut needs_render = true;

    loop {
        let frame_start = Instant::now();

        if termi.should_quit {
            break;
        }

        let is_active = frame_start.duration_since(last_input_at) < Duration::from_secs(2);
        let target_frame_time = if is_active {
            TYPING_FRAME_TIME
        } else {
            IDLE_FRAME_TIME
        };

        if termi.config.show_fps && termi.tracker.fps.update() {
            needs_render = true;
        }

        let timeout = target_frame_time
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    let last_event = termi.last_event;
                    termi.last_event = Some(event);
                    let input_mode = input_handler.resolve_input_mode(&termi);
                    let action = input_handler.handle_input(event, last_event, input_mode);

                    if action == TermiAction::Quit {
                        break;
                    }

                    last_input_at = frame_start;
                    process_action(action, &mut termi);
                    needs_render = true;
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    ..
                }) => {
                    if let Some(action) =
                        handle_click_action(&mut termi, &click_regions, column, row)
                    {
                        process_action(action, &mut termi);
                        last_input_at = frame_start;
                        needs_render = true;
                    }
                }
                Event::Resize(_, _) => {
                    needs_render = true;
                }
                _ => {}
            }
        }

        if frame_start.duration_since(last_time_update) >= Duration::from_millis(100) {
            // check for time completion
            if termi.tracker.should_time_complete() {
                termi.tracker.complete();
                needs_render = true;
            }

            if termi.tracker.update_time_remaining() {
                needs_render = true;
            }

            last_time_update = frame_start;
        }

        if frame_start.duration_since(last_metrics_update) >= Duration::from_millis(500) {
            termi.tracker.update_metrics();
            if termi.tracker.status == Status::Typing && !termi.config.hide_live_wpm {
                needs_render = true;
            }
            last_metrics_update = frame_start;
        }

        // re-render if needed
        if needs_render {
            let fps_display = if termi.config.show_fps {
                Some(termi.tracker.fps.get())
            } else {
                None
            };

            terminal.draw(|frame| {
                click_regions = draw_ui(frame, &mut termi, fps_display);
            })?;

            needs_render = false;
        }

        last_tick = frame_start;
    }

    Ok(())
}
