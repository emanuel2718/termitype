use crate::{
    actions::{self, Action},
    builders::lexicon_builder::Lexicon,
    config::{Config, Mode},
    constants::db_file,
    db::Db,
    db_writer::{DbWriter, EnqueueError},
    error::AppError,
    handler::AppHandler,
    input::{Input, InputContext},
    leaderboard::Leaderboard,
    log_debug, log_error, log_info,
    menu::{Menu, MenuAction},
    modal::Modal,
    notify_error, notify_info,
    perf::PerfMetrics,
    theme,
    tracker::Tracker,
    tui::{self, components::typing_cache::TypingRenderCache},
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use crossterm::execute;
use ratatui::{Terminal, prelude::Backend};
use std::io::stdout;
use std::time::{Duration, Instant};

const MAX_EVENT_BATCH: usize = 256;
const NOTIFICATION_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> anyhow::Result<()> {
    let mut input = Input::new();
    let mut app = App::new(config);

    theme::init_from_config(config)?;
    draw_frame(terminal, &mut app)?;

    log_info!("The config: {config:?}");

    let run_result: anyhow::Result<()> = (|| {
        loop {
            if app.should_quit {
                break;
            }

            let wait_duration = app.next_wait_duration(Instant::now());
            let mut batch_size = 0usize;

            if event::poll(wait_duration)? {
                if let Some(started_at) = process_event_read(&mut input, &mut app, event::read()?)?
                {
                    app.last_event_started_at = Some(started_at);
                    batch_size += 1;
                }

                while batch_size < MAX_EVENT_BATCH && event::poll(Duration::ZERO)? {
                    if let Some(started_at) =
                        process_event_read(&mut input, &mut app, event::read()?)?
                    {
                        app.last_event_started_at = Some(started_at);
                        batch_size += 1;
                    }
                }
                app.perf.on_queue_depth(batch_size.saturating_sub(1));
            }

            post_iteration_updates(&mut app);
            maybe_draw_frame(terminal, &mut app)?;
        }
        Ok(())
    })();

    app.shutdown_workers();
    run_result
}

fn process_event_read(
    input: &mut Input,
    app: &mut App,
    event: Event,
) -> Result<Option<Instant>, AppError> {
    match event {
        // https://ratatui.rs/faq/#why-am-i-getting-duplicate-key-events-on-windows
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            let event_started_at = Instant::now();
            app.perf.on_input_event();
            handle_key_event(input, app, key_event, event_started_at)?;
            Ok(Some(event_started_at))
        }
        Event::Resize(_, _) => {
            app.perf.on_input_event();
            app.mark_high_priority_redraw();
            Ok(Some(Instant::now()))
        }
        _ => Ok(None),
    }
}

fn handle_key_event(
    input: &mut Input,
    app: &mut App,
    key_event: KeyEvent,
    event_started_at: Instant,
) -> Result<(), AppError> {
    app.note_key_input();
    let input_ctx = app.resolve_input_context();
    let input_result = input.handle(key_event, input_ctx);
    if !input_result.skip_debounce && app.handle_debounce() {
        return Ok(());
    }

    if matches!(input_result.action, Action::NoOp) {
        return Ok(());
    }

    actions::handle_action(app, input_result.action)?;
    app.perf.on_action_from_event(event_started_at);
    app.mark_high_priority_redraw();
    Ok(())
}

fn post_iteration_updates(app: &mut App) {
    app.tracker.try_metrics_update();
    if app.tracker.check_completion() {
        app.try_save_results();
        app.mark_high_priority_redraw();
    }

    app.maybe_mark_live_tick_redraw();
    app.maybe_mark_notification_redraw();
    app.perf.maybe_log();
}

fn maybe_draw_frame<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<()> {
    let now = Instant::now();
    if !app.should_draw(now) {
        return Ok(());
    }
    draw_frame(terminal, app)
}

fn draw_frame<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<()> {
    let draw_started_at = app.perf.on_draw_started();
    terminal
        .draw(|frame| {
            let _ = tui::renderer::draw_ui(frame, app);
        })
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    app.on_did_draw();
    app.perf
        .on_draw_completed(draw_started_at, app.last_event_started_at);
    app.last_event_started_at = None;
    Ok(())
}

pub struct App {
    pub db: Option<Db>,
    db_writer: Option<DbWriter>,
    pub config: Config,
    pub menu: Menu,
    pub modal: Option<Modal>,
    pub leaderboard: Option<Leaderboard>,
    pub handler: AppHandler,
    pub lexicon: Lexicon,
    pub tracker: Tracker,
    pub typing_cache: TypingRenderCache,
    pub typing_revision: u64,
    pub perf: PerfMetrics,
    should_quit: bool,
    needs_redraw: bool,
    force_redraw: bool,
    last_draw_at: Instant,
    last_key_input_at: Option<Instant>,
    last_live_ui_tick: Option<u64>,
    last_notification_check_at: Instant,
    last_event_started_at: Option<Instant>,
    last_notification_count: usize,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let lexicon = Lexicon::new(config).unwrap();
        #[allow(unused_mut)]
        let mut tracker = Tracker::new(lexicon.words.clone(), config.current_mode());

        #[cfg(debug_assertions)]
        if config.cli.show_results {
            Self::force_show_results_screen(&mut tracker);
        }

        let db = match Db::new(db_file()) {
            Ok(db) => Some(db),
            Err(err) => {
                log_error!("DB: Failed to initialize local database with: {err}");
                notify_error!("Faled to initialize Local Database");
                None
            }
        };
        let db_writer = if db.is_some() {
            Some(DbWriter::new())
        } else {
            None
        };

        Self {
            db,
            db_writer,
            config: config.clone(),
            menu: Menu::new(),
            modal: None,
            leaderboard: None,
            handler: AppHandler,
            tracker,
            lexicon,
            typing_cache: TypingRenderCache::default(),
            typing_revision: 0,
            perf: PerfMetrics::default(),
            should_quit: false,
            needs_redraw: true,
            force_redraw: true,
            last_draw_at: Instant::now(),
            last_key_input_at: None,
            last_live_ui_tick: None,
            last_notification_check_at: Instant::now(),
            last_event_started_at: None,
            last_notification_count: 0,
        }
    }

    pub fn quit(&mut self) -> Result<(), AppError> {
        self.sync_global_changes()?;
        self.should_quit = true;
        Ok(())
    }

    /// Mark that the app needs to redraw on next iteration
    fn mark_needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    fn mark_high_priority_redraw(&mut self) {
        self.force_redraw = true;
        self.mark_needs_redraw();
    }

    fn note_key_input(&mut self) {
        self.last_key_input_at = Some(Instant::now());
    }

    /// Tracks token-state changes so typing-area cache can invalidate cheaply.
    pub fn bump_typing_revision(&mut self) {
        self.typing_revision = self.typing_revision.wrapping_add(1);
        self.typing_cache.invalidate();
        self.mark_needs_redraw();
    }

    fn target_frame_duration(&self) -> Duration {
        match self.resolve_input_context() {
            InputContext::Typing => Duration::from_millis(16), // ~60fps
            InputContext::Menu { .. } | InputContext::Modal | InputContext::Leaderboard => {
                Duration::from_millis(16)
            }
            InputContext::Idle => Duration::from_millis(66), // ~15fps
            InputContext::Completed => Duration::from_millis(100), // ~10fps
        }
    }

    fn should_draw(&self, now: Instant) -> bool {
        if !self.needs_redraw {
            return false;
        }

        if self.force_redraw {
            return true;
        }

        now.saturating_duration_since(self.last_draw_at) >= self.target_frame_duration()
    }

    fn on_did_draw(&mut self) {
        self.needs_redraw = false;
        self.force_redraw = false;
        self.last_draw_at = Instant::now();
    }

    fn next_wait_duration(&self, now: Instant) -> Duration {
        let poll = self.get_poll_duration();
        if !self.needs_redraw || self.force_redraw {
            return if self.force_redraw {
                Duration::ZERO
            } else {
                poll
            };
        }

        let frame_wait = self
            .target_frame_duration()
            .saturating_sub(now.saturating_duration_since(self.last_draw_at));
        frame_wait.min(poll)
    }

    fn maybe_mark_live_tick_redraw(&mut self) {
        if !self.tracker.is_typing() || self.tracker.is_complete() {
            self.last_live_ui_tick = None;
            return;
        }

        let input_idle_for = self
            .last_key_input_at
            .map(|last| last.elapsed())
            .unwrap_or(Duration::MAX);

        // If keys are arriving rapidly, key-driven redraws are already frequent enough.
        if input_idle_for < Duration::from_millis(150) {
            return;
        }

        // 4Hz shortly after typing slows down, 2Hz when idle longer.
        let tick_ms = if input_idle_for >= Duration::from_millis(800) {
            500
        } else {
            250
        };
        let current_tick = (self.tracker.elapsed_time().as_millis() / tick_ms) as u64;
        if self.last_live_ui_tick != Some(current_tick) {
            self.last_live_ui_tick = Some(current_tick);
            self.mark_needs_redraw();
        }
    }

    fn shutdown_workers(&mut self) {
        if let Some(writer) = self.db_writer.as_mut() {
            writer.shutdown();
        }
        self.db_writer = None;
    }

    fn maybe_mark_notification_redraw(&mut self) {
        if self.config.should_hide_notifications() {
            return;
        }

        if self.last_notification_check_at.elapsed() < NOTIFICATION_POLL_INTERVAL {
            return;
        }
        self.last_notification_check_at = Instant::now();

        let current_count = crate::notifications::count();
        if current_count != self.last_notification_count {
            self.mark_needs_redraw();
            self.last_notification_count = current_count;
        }
    }

    /// Get the appropriate poll duration based on current state
    fn get_poll_duration(&self) -> Duration {
        let ctx = self.resolve_input_context();
        match ctx {
            InputContext::Typing => Duration::from_millis(8),
            InputContext::Menu { .. } | InputContext::Modal | InputContext::Leaderboard => {
                Duration::from_millis(16)
            }
            InputContext::Idle => Duration::from_millis(50),
            InputContext::Completed => Duration::from_millis(100),
        }
    }

    pub fn redo(&mut self) -> Result<(), AppError> {
        self.tracker
            .reset(self.lexicon.words.clone(), self.config.current_mode());
        self.bump_typing_revision();
        Ok(())
    }

    pub fn restart(&mut self) -> Result<(), AppError> {
        // NOTE: if we start a new test we want to clear the custom words flag as starting a new
        //       test is designed to generate a completely new test. If the user want to keep
        //       the custom words then a `Redo` is the option.
        // if self.config.cli.words.is_some() {
        //     self.config.cli.clear_custom_words_flag();
        // }
        self.lexicon.regenerate(&self.config)?;
        self.tracker
            .reset(self.lexicon.words.clone(), self.config.current_mode());
        self.bump_typing_revision();
        Ok(())
    }

    pub fn try_save_results(&mut self) {
        if !self.config.can_save_results() {
            // QUESTION: should we notify here that we are not storing the results due to the option of `no_save`?
            log_info!("DB: Not saving test result to local database due to `--no-save` flag");
            return;
        }

        if !self.should_save_results() {
            notify_info!("Test invalid - too short")
        }

        let mut result = Db::build_result(&self.config, &self.tracker);

        if let Some(writer) = self.db_writer.as_ref() {
            match writer.enqueue(result) {
                Ok(()) => return,
                Err(EnqueueError::Full(r)) => {
                    log_debug!("DB writer queue is full, falling back to sync write");
                    result = r;
                }
                Err(EnqueueError::Disconnected(r)) => {
                    log_error!("DB writer disconnected, falling back to sync write");
                    self.db_writer = None;
                    result = r;
                }
            }
        }

        let Some(db) = &mut self.db else {
            log_debug!("DB: No database availabe, skipping saving results");
            notify_error!("Could not save results");
            return;
        };

        if let Err(err) = db.write_result(result) {
            log_error!("DB: Failed trying to save results with error: {err}");
            notify_error!("Could not save results")
        };
    }

    fn should_save_results(&self) -> bool {
        const MIN_TIME_FOR_SAVING: usize = if cfg!(debug_assertions) { 1 } else { 15 };
        const MIN_WORDS_FOR_SAVING: usize = if cfg!(debug_assertions) { 1 } else { 10 };
        match self.config.current_mode() {
            Mode::Time(duration) => duration >= MIN_TIME_FOR_SAVING,
            Mode::Words(count) => count >= MIN_WORDS_FOR_SAVING,
        }
    }

    // TODO: do this cleanly
    pub(crate) fn try_preview(&mut self) -> Result<(), AppError> {
        let is_theme_preview = self
            .menu
            .current_item()
            .map(|item| {
                item.has_preview && matches!(item.action, MenuAction::Action(Action::SetTheme(_)))
            })
            .unwrap_or(false);

        if !is_theme_preview {
            theme::cancel_theme_preview();
        }

        if let Some(item) = self.menu.current_item()
            && item.has_preview
        {
            match &item.action {
                MenuAction::Action(Action::SetTheme(name)) => theme::set_as_preview_theme(name),
                MenuAction::Action(Action::SetCursorVariant(variant)) => {
                    let _ = execute!(stdout(), variant.to_crossterm());
                    Ok(())
                }
                _ => Ok(()),
            }?;
        }
        Ok(())
    }

    pub(crate) fn restore_cursor_style(&self) {
        use crossterm::execute;
        use std::io::stdout;

        let current_variant = self.config.current_cursor_variant();
        let _ = execute!(stdout(), current_variant.to_crossterm());
    }

    fn sync_global_changes(&mut self) -> Result<(), AppError> {
        // NOTE: sync the theme changes before quitting.
        let theme = theme::current_theme();
        log_debug!("The current theme: {theme:?}");
        self.config.change_theme(theme.as_ref());
        self.config.persist()?;
        Ok(())
    }

    // NOTE(ema): this is order dependet which can be dangerous and confusing.
    // For example, if we put the modal `if` after the menu check it will never reach the modal if
    // we opened the modal from the menu (as in this case we, currently, keep the menu open.
    fn resolve_input_context(&self) -> InputContext {
        if self.modal.is_some() {
            InputContext::Modal
        } else if self.leaderboard.as_ref().is_some_and(|l| l.is_open()) {
            InputContext::Leaderboard
        } else if self.menu.is_open() {
            InputContext::Menu {
                searching: self.menu.is_searching(),
            }
        } else if self.tracker.is_complete() {
            InputContext::Completed
        } else if self.tracker.in_progress() {
            InputContext::Typing
        } else {
            InputContext::Idle
        }
    }

    fn handle_debounce(&self) -> bool {
        if self.tracker.is_complete()
            && let Some(end_time) = self.tracker.end_time
            && end_time.elapsed() < Duration::from_millis(500)
        {
            return true;
        }
        false
    }

    #[cfg(debug_assertions)]
    fn force_show_results_screen(tracker: &mut Tracker) {
        tracker.start_typing();
        let test_chars = "hello world test";
        for c in test_chars.chars() {
            let _ = tracker.type_char(c);
        }
        tracker.complete();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, tracker::TypingStatus};

    #[test]
    fn test_command_palette_open_pause_resume() {
        let config = Config::default();
        let mut app = App::new(&config);
        app.config.change_mode(Mode::with_words(2)).unwrap();

        app.handler.handle_input(&mut app, 'a').unwrap();
        app.handler.handle_input(&mut app, 'n').unwrap();
        app.handler.handle_input(&mut app, 'o').unwrap();
        app.handler.handle_input(&mut app, 't').unwrap();
        app.handler.handle_input(&mut app, 'h').unwrap();
        app.handler.handle_input(&mut app, 'e').unwrap();
        app.handler.handle_input(&mut app, 'r').unwrap();

        app.handler.handle_command_palette_open(&mut app).unwrap();
        app.handler.handle_command_palette_open(&mut app).unwrap();
        app.handler
            .handle_menu_update_search(&mut app, "s".to_string())
            .unwrap();
        assert!(app.tracker.is_paused());

        // if we are in the command palette we can close by hitting `Esc`,
        // and hitting `Esc` while searching a menu will trigger `Action::MenuExitSearch`
        app.handler.handle_menu_exit_search(&mut app).unwrap();
        assert!(app.tracker.is_resuming());

        app.handler.handle_input(&mut app, ' ').unwrap();
    }

    #[test]
    fn test_toggling_leaderboard_should_pause_game() {
        let config = Config::default();
        let mut app = App::new(&config);
        app.tracker.start_typing();
        app.tracker.type_char('h').unwrap();

        assert_eq!(app.tracker.status, TypingStatus::InProgress);
        app.handler.handle_leaderboard_toggle(&mut app).unwrap();

        assert_eq!(app.tracker.status, TypingStatus::Paused);
        app.handler.handle_leaderboard_toggle(&mut app).unwrap();
        assert_eq!(app.tracker.status, TypingStatus::Resuming);
        app.tracker.type_char('i').unwrap();
        assert_eq!(app.tracker.status, TypingStatus::InProgress);
    }
}
