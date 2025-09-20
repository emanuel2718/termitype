use crate::{
    actions,
    builders::lexicon_builder::Lexicon,
    config::Config,
    error::AppError,
    input::{Input, InputContext},
    log_debug, log_info,
    menu::{Menu, MenuContext, MenuMotion},
    theme,
    tracker::Tracker,
    tui,
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{prelude::Backend, Terminal};
use std::time::{Duration, Instant};

pub struct App {
    pub config: Config,
    pub menu: Menu,
    pub lexicon: Lexicon,
    pub tracker: Tracker,
    should_quit: bool,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let lexicon = Lexicon::new(config).unwrap();
        let mut tracker = Tracker::new(lexicon.words.clone(), config.current_mode());

        #[cfg(debug_assertions)]
        if config.cli.show_results {
            Self::force_show_results_screen(&mut tracker);
        }

        Self {
            config: config.clone(),
            menu: Menu::new(),
            tracker,
            lexicon,
            should_quit: false,
        }
    }

    pub fn quit(&mut self) -> Result<(), AppError> {
        self.sync_global_changes()?;
        self.should_quit = true;
        Ok(())
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

    pub fn handle_input(&mut self, chr: char) -> Result<(), AppError> {
        if self.tracker.is_complete() {
            return Ok(());
        }
        match self.tracker.type_char(chr) {
            Ok(()) => Ok(()),
            Err(AppError::IllegalSpaceCharacter) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn handle_menu_open(&mut self, ctx: MenuContext) -> Result<(), AppError> {
        self.menu.open(ctx, &self.config)?;
        self.try_preview()?;
        self.tracker.toggle_pause();
        Ok(())
    }

    pub fn handle_menu_close(&mut self) -> Result<(), AppError> {
        // TODO: this clearing of preview should be done cleanly
        theme::cancel_theme_preview();
        self.menu.close()?;
        self.tracker.toggle_pause();
        Ok(())
    }

    pub fn handle_menu_backtrack(&mut self) -> Result<(), AppError> {
        // TODO: this clearing of preview should be done cleanly
        theme::cancel_theme_preview();
        self.menu.back()?;
        if !self.menu.is_open() {
            self.tracker.toggle_pause();
        }
        Ok(())
    }

    pub fn handle_menu_navigate(&mut self, motion: MenuMotion) -> Result<(), AppError> {
        self.menu.navigate(motion);
        self.try_preview()?;
        Ok(())
    }

    pub fn handle_menu_select(&mut self) -> Result<(), AppError> {
        if let Ok(Some(action)) = self.menu.select(&self.config) {
            actions::handle_action(self, action)?;
            // note: the action above could've been a menu closing action.
            if !self.menu.is_open() {
                theme::cancel_theme_preview();
                self.tracker.toggle_pause();
            }
        }
        Ok(())
    }

    pub fn handle_menu_exit_search(&mut self) -> Result<(), AppError> {
        self.menu.exit_search();
        Ok(())
    }

    pub fn handle_menu_backspace_search(&mut self) -> Result<(), AppError> {
        if !self.menu.search_query().is_empty() {
            let mut query = self.menu.search_query().to_string();
            query.pop();
            if query.is_empty() {
                self.menu.exit_search();
            } else {
                self.menu.update_search(query);
            }
            self.try_preview()?
        }
        Ok(())
    }

    pub fn handle_menu_init_search(&mut self) -> Result<(), AppError> {
        self.menu.init_search();
        Ok(())
    }

    pub fn handle_menu_update_search(&mut self, query: String) -> Result<(), AppError> {
        if query.is_empty() {
            return Ok(()); // TODO: this is dumb
        }
        let current_query = self.menu.search_query().to_string();
        let new_query = format!("{}{}", current_query, query);
        self.menu.update_search(new_query);
        self.try_preview()?;

        Ok(())
    }

    pub fn handle_backspace(&mut self) -> Result<(), AppError> {
        match self.tracker.backspace() {
            Ok(()) => Ok(()),
            Err(AppError::TypingTestNotInProgress) => Ok(()),
            Err(AppError::IllegalBackspace) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn handle_change_line_count(&mut self) -> Result<(), AppError> {
        // TODO: eventually do this corrrectly through a modal or something, just messgin around atm.
        //       this will need to receive the desired visible line count....
        const MAX_LINE_COUNT: u8 = 6;
        let current = self.config.current_line_count();
        let new_count = if current >= MAX_LINE_COUNT {
            1
        } else {
            current + 1
        };
        self.config.change_visible_lines_count(new_count);
        Ok(())
    }

    // TODO: do this cleanly
    fn try_preview(&mut self) -> Result<(), AppError> {
        if let Some(menu) = self.menu.current_menu() {
            if let Some(item) = self.menu.current_item() {
                if item.has_preview {
                    match menu.ctx {
                        MenuContext::Themes => theme::set_as_preview_theme(item.label().as_str()),
                        _ => Ok(()),
                    }?;
                }
            }
        };
        Ok(())
    }

    fn sync_global_changes(&mut self) -> Result<(), AppError> {
        // NOTE: sync the theme changes before quitting.
        let theme = theme::current_theme();
        log_debug!("The current theme: {theme:?}");
        self.config.change_theme(theme);
        self.config.persist()?;
        Ok(())
    }

    fn resolve_input_context(&self) -> InputContext {
        if self.menu.is_open() {
            InputContext::Menu {
                searching: self.menu.is_searching(),
            }
        } else if self.tracker.is_complete() {
            InputContext::Completed
        } else if self.tracker.is_typing() {
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

    fn force_show_results_screen(tracker: &mut Tracker) {
        tracker.start_typing();
        let test_chars = "hello world test";
        for c in test_chars.chars() {
            let _ = tracker.type_char(c);
        }
        tracker.complete();
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> anyhow::Result<()> {
    let mut input = Input::new();
    let mut app = App::new(config);

    theme::init_from_config(config)?;

    log_info!("The config: {config:?}");
    loop {
        if app.should_quit {
            break;
        }
        if event::poll(Duration::from_millis(75))? {
            match event::read()? {
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    // TODO: resolve input contxt
                    let input_ctx = app.resolve_input_context();
                    let input_result = input.handle(event, input_ctx);
                    if !input_result.skip_debounce && app.handle_debounce() {
                        continue;
                    }
                    actions::handle_action(&mut app, input_result.action)?;
                }
                _ => {}
            }
        }

        app.tracker.try_metrics_update();
        app.tracker.check_completion();

        terminal.draw(|frame| {
            // TODO: return the click actions
            let _ = tui::renderer::draw_ui(frame, &mut app);
        })?;
    }

    Ok(())
}
