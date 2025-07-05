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
    db::TermiDB,
    input::InputHandler,
    leaderboard::Leaderboard,
    log_debug, log_error,
    menu::MenuState,
    modal::InputModal,
    styles::ResultsStyle,
    theme::Theme,
    tracker::{Status, Tracker},
    ui::{components::elements::TermiClickableRegions, draw_ui},
};

pub struct Termi {
    pub db: Option<TermiDB>,
    pub config: Config,
    pub tracker: Tracker,
    pub theme: Theme,
    pub builder: Builder,
    pub words: String,
    pub menu: MenuState,
    pub modal: Option<InputModal>,
    pub leaderboard: Option<Leaderboard>,
    pub preview_theme: Option<Theme>,
    pub preview_cursor: Option<SetCursorStyle>,
    pub preview_ascii_art: Option<String>,
    pub preview_results_style: Option<ResultsStyle>,
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
            .field("preview_results_style", &self.preview_results_style)
            .field("builder", &self.builder)
            .field("words", &self.words)
            .field("menu", &self.menu)
            .field("leaderboard", &self.leaderboard)
            .field("modal", &self.modal);
        debug_struct.finish()
    }
}

impl Termi {
    pub fn new(config: &Config) -> Self {
        let mut builder = Builder::new();
        let words = builder.generate_test(config);
        let db = match config.no_track {
            true => None,
            false => match TermiDB::new() {
                Ok(db) => Some(db),
                Err(err) => {
                    log_error!("DB: Failed to initialize database: {err}");
                    None
                }
            },
        };

        let leaderboard = match config.no_track {
            true => None,
            false => Some(Leaderboard::new()),
        };

        Self {
            db,
            leaderboard,
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
            preview_results_style: None,
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

    pub fn current_results_style(&self) -> ResultsStyle {
        self.preview_results_style
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.config.resolve_results_style())
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

    pub fn save_results(&mut self) -> bool {
        if self.tracker.status != Status::Completed {
            log_debug!("Attempted to save incomplete test result");
            return false;
        }

        if !self.should_save_results() {
            log_debug!("Test does not meet the minimum requirements for saving results");
            return false;
        }

        let Some(db) = &mut self.db else {
            log_debug!("No database available, skipping result save");
            return false;
        };

        let is_high_score = db.is_high_score(&self.config, self.tracker.wpm);
        if is_high_score {
            self.tracker.mark_high_score();
        }

        if let Err(err) = db.write(&self.config, &self.tracker) {
            log_error!("DB: Failed to save test results: {err}");
            return false;
        }
        true
    }

    /// Checks if the test meets the minimum requirements for saving the test results
    fn should_save_results(&self) -> bool {
        const MIN_TIME_FOR_SAVING: u64 = 15;
        const MIN_WORDS_FOR_SAVING: usize = 10;
        match self.config.current_mode() {
            crate::config::Mode::Time { duration } => duration >= MIN_TIME_FOR_SAVING,
            crate::config::Mode::Words { count } => count >= MIN_WORDS_FOR_SAVING,
        }
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

    if config.reset_db {
        if let Some(db) = &termi.db {
            let items_deleted = db.reset();
            if let Err(err) = items_deleted {
                log_error!("Something went wrong calling db.reset: {err}")
            }
            log_debug!("Should reset the database");
        } else {
            log_error!("Trying to reset database without having one in the first place");
        };
    }

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
                termi.save_results();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test_env() -> TempDir {
        let _guard = ENV_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        if cfg!(target_os = "macos") {
            std::env::set_var("HOME", &temp_path);
        } else if cfg!(target_os = "windows") {
            std::env::set_var("APPDATA", &temp_path);
        } else {
            std::env::set_var("XDG_CONFIG_HOME", &temp_path);
        }

        temp_dir
    }

    fn create_config_with_tracking(no_track: bool) -> Config {
        let mut config = Config::default();
        config.no_track = no_track;
        config
    }

    #[test]
    fn test_termi_with_tracking_enabled() {
        let _ = setup_test_env();
        let config = Config::default();
        let mut termi = Termi::new(&config);

        assert!(termi.db.is_some());
        assert!(termi.leaderboard.is_some());

        termi.tracker.status = Status::Completed;
        termi.tracker.completion_time = Some(30.0);
        termi.tracker.wpm = 50.0;

        termi.save_results();

        assert!(termi.db.is_some())
    }

    #[test]
    fn test_termi_with_tracking_disabled() {
        let _ = setup_test_env();
        let mut config = Config::default();
        config.no_track = true;
        let mut termi = Termi::new(&config);

        assert!(termi.db.is_none());
        assert!(termi.leaderboard.is_none());

        termi.tracker.status = Status::Completed;
        termi.tracker.completion_time = Some(30.0);
        termi.tracker.wpm = 50.0;

        assert!(!termi.save_results());

        assert!(termi.db.is_none());
    }

    #[test]
    fn test_save_results_incomplete_test() {
        let _temp = setup_test_env();
        let config = create_config_with_tracking(false);
        let mut termi = Termi::new(&config);

        termi.tracker.status = Status::Typing;
        termi.tracker.wpm = 50.0;

        assert!(!termi.save_results());

        assert_eq!(termi.tracker.status, Status::Typing);
    }

    #[test]
    fn test_should_save_results_time_mode() {
        let _temp = setup_test_env();
        let mut config = create_config_with_tracking(false);
        // we only save results for times greater than 15 seconds

        config.time = Some(15);
        let termi = Termi::new(&config);
        assert!(termi.should_save_results());

        config.time = Some(10);
        let termi = Termi::new(&config);
        assert!(!termi.should_save_results());
    }

    #[test]
    fn test_should_save_results_words_mode() {
        let _temp = setup_test_env();
        let mut config = create_config_with_tracking(false);
        // we only save results for tests with more than 10 words

        config.time = None;
        config.word_count = Some(10);
        let termi = Termi::new(&config);
        assert!(termi.should_save_results());

        config.word_count = Some(5);
        let termi = Termi::new(&config);
        assert!(!termi.should_save_results());
    }

    #[test]
    fn test_configuration_flag_consistency() {
        let _temp = setup_test_env();

        let config_with_tracking = create_config_with_tracking(false);
        let config_no_tracking = create_config_with_tracking(true);

        assert!(!config_with_tracking.no_track, "Tracking should be enabled");
        assert!(config_no_tracking.no_track, "Tracking should be disabled");

        let termi_with_tracking = Termi::new(&config_with_tracking);
        let termi_no_tracking = Termi::new(&config_no_tracking);

        assert!(termi_with_tracking.leaderboard.is_some());
        assert!(termi_no_tracking.db.is_none() && termi_no_tracking.leaderboard.is_none());
    }

    #[test]
    fn test_database_reset_with_no_database() {
        let _temp = setup_test_env();
        let config = create_config_with_tracking(true);
        let termi = Termi::new(&config);

        if let Some(db) = &termi.db {
            let _items_deleted = db.reset();
            panic!("Should not reach here when database is None");
        }
    }

    #[test]
    fn test_database_reset_with_database_available() {
        let _temp = setup_test_env();
        let config = create_config_with_tracking(false);
        let termi = Termi::new(&config);

        if let Some(db) = &termi.db {
            let items_deleted = db.reset();
            assert!(items_deleted.is_ok());
        }
    }
}
