use crate::{
    actions::{self},
    builders::lexicon_builder::Lexicon,
    config::Config,
    error::AppError,
    input::{Input, InputContext},
    log_debug, log_info,
    menu::Menu,
    theme,
    tracker::Tracker,
    tui,
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{prelude::Backend, Terminal};
use std::time::Duration;

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
        let tracker = Tracker::new(lexicon.words.clone(), config.current_mode());

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
        if self.config.cli.words.is_some() {
            self.config.cli.clear_custom_words_flag();
        }
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
            Ok(()) => {
                self.tracker.start_typing();
                Ok(())
            }
            Err(AppError::IllegalSpaceCharacter) => Ok(()),
            Err(e) => Err(e),
        }
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
        } else {
            InputContext::Typing
        }
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
                    let action = input.handle(event, input_ctx);
                    actions::handle_action(&mut app, action)?;
                }
                _ => {}
            }
        }

        terminal.draw(|frame| {
            // TODO: return the click actions
            let _ = tui::renderer::draw_ui(frame, &mut app);
        })?;

        app.tracker.check_completion();
    }

    Ok(())
}
