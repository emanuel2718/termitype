use crate::{
    actions::{self, Action},
    builders::lexicon_builder::Lexicon,
    config::{Config, Mode},
    constants::db_file,
    db::Db,
    error::AppError,
    handler::AppHandler,
    input::{Input, InputContext},
    leaderboard::Leaderboard,
    log_debug, log_error, log_info,
    menu::{Menu, MenuAction},
    modal::Modal,
    notify_error, notify_info, theme,
    tracker::Tracker,
    tui,
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use ratatui::{prelude::Backend, Terminal};
use std::io::stdout;
use std::time::Duration;

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> anyhow::Result<()> {
    let mut input = Input::new();
    let mut app = App::new(config);

    theme::init_from_config(config)?;

    // NOTE(ema): this initial draw is needed do to the optimizations around reducing cpu usage on IDLE.
    // These optmizations caused the first draw to happen after `250ms` which felt incredibly sluggish.
    // To work around this, a good quick and easy solution is to do an immediate draw before
    // entering the loop. Probably there's a better way to do this. If future me see this comment...
    // you are probably thinking: "Who the f*k did this? What a sub-optimal way to handle this".
    // ...It was you, always has been
    terminal.draw(|frame| {
        let _ = tui::renderer::draw_ui(frame, &mut app);
    })?;
    app.needs_redraw = false;

    log_info!("The config: {config:?}");
    loop {
        if app.should_quit {
            break;
        }

        let poll_duration = app.get_poll_duration();

        if event::poll(poll_duration)? {
            match event::read()? {
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    let input_ctx = app.resolve_input_context();
                    let input_result = input.handle(event, input_ctx);
                    if !input_result.skip_debounce && app.handle_debounce() {
                        continue;
                    }
                    actions::handle_action(&mut app, input_result.action)?;
                    app.mark_needs_redraw();
                }
                Event::Resize(_, _) => {
                    app.mark_needs_redraw();
                }
                _ => {}
            }
        }

        app.tracker.try_metrics_update();
        if app.tracker.check_completion() {
            app.try_save_results();
            app.mark_needs_redraw();
        }

        if app.tracker.is_typing() {
            app.mark_needs_redraw();
        }

        // if the # of active notification changes we must trigger a redraw, otherwise we end up
        // we infinite duration notification in results  (we don't trigger redraws in `Results`
        // until a `KeyEvent` or `Action`). This is easiest solution to that problem.
        let current_count = crate::notifications::count();
        if current_count != app.last_notification_count {
            log_debug!("Notification count changed, trigger redraw!");
            app.mark_needs_redraw();
            app.last_notification_count = current_count;
        }

        if app.take_needs_redraw() {
            terminal.draw(|frame| {
                // TODO: return the click actions
                let _ = tui::renderer::draw_ui(frame, &mut app);
            })?;
        }
    }

    Ok(())
}

pub struct App {
    pub db: Option<Db>,
    pub config: Config,
    pub menu: Menu,
    pub modal: Option<Modal>,
    pub leaderboard: Option<Leaderboard>,
    pub handler: AppHandler,
    pub lexicon: Lexicon,
    pub tracker: Tracker,
    should_quit: bool,
    needs_redraw: bool,
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

        Self {
            db,
            config: config.clone(),
            menu: Menu::new(),
            modal: None,
            leaderboard: None,
            handler: AppHandler,
            tracker,
            lexicon,
            should_quit: false,
            needs_redraw: true,
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

    /// Check if redraw is needed and consume it
    fn take_needs_redraw(&mut self) -> bool {
        let needs = self.needs_redraw;
        self.needs_redraw = false;
        needs
    }

    /// Get the appropriate poll duration based on current state
    fn get_poll_duration(&self) -> Duration {
        let ctx = self.resolve_input_context();
        match ctx {
            InputContext::Typing => Duration::from_millis(75),
            InputContext::Menu { .. } | InputContext::Modal | InputContext::Leaderboard => {
                Duration::from_millis(100)
            }
            InputContext::Idle => Duration::from_millis(250),
            InputContext::Completed => Duration::from_millis(1000),
        }
    }

    pub fn redo(&mut self) -> Result<(), AppError> {
        self.tracker
            .reset(self.lexicon.words.clone(), self.config.current_mode());
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

        let Some(db) = &mut self.db else {
            log_debug!("DB: No database availabe, skipping saving results");
            notify_error!("Could not save results");
            return;
        };

        // TODO: check for high scores

        if let Err(err) = db.write(&self.config, &self.tracker) {
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

        if let Some(item) = self.menu.current_item() {
            if item.has_preview {
                match &item.action {
                    MenuAction::Action(Action::SetTheme(name)) => theme::set_as_preview_theme(name),
                    MenuAction::Action(Action::SetCursorVariant(variant)) => {
                        let _ = execute!(stdout(), variant.to_crossterm());
                        Ok(())
                    }
                    _ => Ok(()),
                }?;
            }
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
        self.config.change_theme(theme);
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
        if self.tracker.is_complete() {
            if let Some(end_time) = self.tracker.end_time {
                if end_time.elapsed() < Duration::from_millis(500) {
                    return true;
                }
            }
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
    use crate::config::Config;

    #[test]
    fn test_command_palette_pause_resume() {
        let config = Config::default();
        let mut app = App::new(&config);
        app.config
            .change_mode(crate::config::Mode::with_words(2))
            .unwrap();

        app.handler.handle_input(&mut app, 'a').unwrap();
        app.handler.handle_input(&mut app, 'n').unwrap();
        app.handler.handle_input(&mut app, 'o').unwrap();
        app.handler.handle_input(&mut app, 't').unwrap();
        app.handler.handle_input(&mut app, 'h').unwrap();
        app.handler.handle_input(&mut app, 'e').unwrap();
        app.handler.handle_input(&mut app, 'r').unwrap();

        app.handler.handle_command_palette_toggle(&mut app).unwrap();
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
}
