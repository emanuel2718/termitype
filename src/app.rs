use crate::{
    actions::{self},
    builders::lexicon_builder::Lexicon,
    config::{self, Config, Mode, Setting},
    error::AppError,
    input::{Input, InputContext},
    log_debug, log_info,
    menu::{Menu, MenuContext, MenuMotion},
    modal::{Modal, ModalContext},
    theme,
    tracker::Tracker,
    tui,
    variants::{CursorVariant, PickerVariant, ResultsVariant},
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{prelude::Backend, Terminal};
use std::time::Duration;

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

pub struct App {
    pub config: Config,
    pub menu: Menu,
    pub modal: Option<Modal>,
    pub lexicon: Lexicon,
    pub tracker: Tracker,
    should_quit: bool,
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

        Self {
            config: config.clone(),
            menu: Menu::new(),
            modal: None,
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

    pub fn handle_toggle_setting(&mut self, setting: Setting) -> Result<(), AppError> {
        self.config.toggle(setting)?;
        self.restart()?;
        Ok(())
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
        self.restore_cursor_style();
        self.menu.close()?;
        self.tracker.toggle_pause();
        Ok(())
    }

    pub fn handle_menu_toggle(&mut self) -> Result<(), AppError> {
        if self.menu.is_open() {
            return self.handle_menu_close();
        }
        self.handle_menu_open(MenuContext::Root)
    }

    pub fn handle_menu_backtrack(&mut self) -> Result<(), AppError> {
        // TODO: this clearing of preview should be done cleanly
        theme::cancel_theme_preview();
        self.menu.back()?;
        if !self.menu.is_open() {
            self.restore_cursor_style();
            self.tracker.toggle_pause();
        }
        Ok(())
    }

    pub fn handle_menu_navigate(&mut self, motion: MenuMotion) -> Result<(), AppError> {
        self.menu.navigate(motion);
        self.try_preview()?;
        Ok(())
    }

    pub fn handle_menu_shortcut(&mut self, shortcut: char) -> Result<(), AppError> {
        if let Some(menu) = self.menu.current_menu_mut() {
            if let Some((idx, _)) = menu.find_by_shortcut(shortcut) {
                menu.set_current_index(idx);
                return self.handle_menu_select();
            }
        }
        Ok(())
    }

    pub fn handle_menu_select(&mut self) -> Result<(), AppError> {
        if let Ok(Some(action)) = self.menu.select(&self.config) {
            actions::handle_action(self, action)?;
            // note: the action above could've been a menu closing action.
            if !self.menu.is_open() {
                theme::cancel_theme_preview();
                self.restore_cursor_style();
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
        self.menu.backspace_search();
        // self.try_preview()?;
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

    pub fn handle_modal_open(&mut self, ctx: ModalContext) -> Result<(), AppError> {
        self.modal = Some(Modal::new(ctx));
        Ok(())
    }

    pub fn handle_modal_close(&mut self) -> Result<(), AppError> {
        self.modal = None;
        Ok(())
    }

    pub fn handle_modal_backspace(&mut self) -> Result<(), AppError> {
        if let Some(modal) = self.modal.as_mut() {
            modal.handle_backspace();
        }
        Ok(())
    }

    pub fn handle_modal_input(&mut self, chr: char) -> Result<(), AppError> {
        if let Some(modal) = self.modal.as_mut() {
            modal.handle_input(chr);
        }
        Ok(())
    }

    pub fn handle_modal_confirm(&mut self) -> Result<(), AppError> {
        if let Some(modal) = self.modal.as_mut() {
            // NOTE(ema): this would've been so clean, but unfortunately we don't know wich context
            // we currently at in `keymap_builder`. To what we would need to map `Action::ModalConfirm` to?
            // Maybe in the future i've grinded enough intellect xp to be able to tackle this
            // actions::handle_action(self, action);

            match modal.ctx {
                ModalContext::CustomTime => {
                    // TODO: find a cleaner way of doing this. Maybe have get_value handle the parsing inside?
                    if let Ok(val) = modal.get_value() {
                        if let Ok(secs) = val.parse::<usize>() {
                            self.config.change_mode(Mode::with_time(secs))?;
                            self.restart()?
                        }
                    }
                }
                ModalContext::CustomWordCount => {
                    // TODO: find a cleaner way of doing this. Maybe have get_value handle the parsing inside?
                    if let Ok(val) = modal.get_value() {
                        if let Ok(count) = val.parse::<usize>() {
                            self.config.change_mode(Mode::with_words(count))?;
                            self.restart()?
                        }
                    }
                }
                ModalContext::CustomLineCount => {
                    // TODO: find a cleaner way of doing this. Maybe have get_value handle the parsing inside?
                    if let Ok(val) = modal.get_value() {
                        if let Ok(count) = val.parse::<u8>() {
                            self.config.change_visible_lines_count(count);
                            self.restart()?
                        }
                    }
                }
                ModalContext::ExitConfirmation => self.quit()?,
            }
        }
        self.handle_modal_close()?;
        self.handle_menu_close()?;
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> Result<(), AppError> {
        match self.tracker.backspace() {
            Ok(()) => Ok(()),
            Err(AppError::TypingTestNotInProgress) => Ok(()),
            Err(AppError::IllegalBackspace) => Ok(()),
            Err(AppError::IllegalSpaceCharacter) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn handle_set_line_count(&mut self, line_count: u8) -> Result<(), AppError> {
        self.config.change_visible_lines_count(line_count);
        Ok(())
    }

    pub fn handle_set_cursor(&mut self, variant: CursorVariant) -> Result<(), AppError> {
        self.config.change_cursor_variant(variant);
        // self.restart()?;
        Ok(())
    }

    pub fn handle_set_picker(&mut self, variant: PickerVariant) -> Result<(), AppError> {
        self.config.change_picker_variant(variant);
        Ok(())
    }

    pub fn handle_set_result(&mut self, variant: ResultsVariant) -> Result<(), AppError> {
        self.config.change_results_variant(variant);
        Ok(())
    }

    pub fn handle_set_time(&mut self, secs: usize) -> Result<(), AppError> {
        self.config.change_mode(config::Mode::with_time(secs))?;
        self.restart()?;
        Ok(())
    }

    pub fn handle_set_words(&mut self, count: usize) -> Result<(), AppError> {
        self.config.change_mode(config::Mode::with_words(count))?;
        self.restart()?;
        Ok(())
    }

    pub fn handle_set_language(&mut self, lang: String) -> Result<(), AppError> {
        self.config.change_language(lang);
        self.restart()?;
        Ok(())
    }

    pub fn handle_set_ascii_art(&mut self, art: String) -> Result<(), AppError> {
        self.config.change_ascii_art(art);
        Ok(())
    }

    // TODO: do this cleanly
    fn try_preview(&mut self) -> Result<(), AppError> {
        if let Some(menu) = self.menu.current_menu() {
            if let Some(item) = self.menu.current_item() {
                if item.has_preview {
                    match menu.ctx {
                        MenuContext::Themes => theme::set_as_preview_theme(item.label().as_str()),
                        MenuContext::Cursor => {
                            use crate::actions::Action;
                            use crate::menu::MenuAction;
                            use crossterm::execute;
                            use std::io::stdout;

                            if let MenuAction::Action(Action::SetCursorVariant(variant)) =
                                &item.action
                            {
                                let _ = execute!(stdout(), variant.to_crossterm());
                            }
                            Ok(())
                        }
                        _ => Ok(()),
                    }?;
                }
            }
        };
        Ok(())
    }

    fn restore_cursor_style(&self) {
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
    // TODO: improve this
    fn resolve_input_context(&self) -> InputContext {
        if self.modal.is_some() {
            InputContext::Modal
        } else if self.menu.is_open() {
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
