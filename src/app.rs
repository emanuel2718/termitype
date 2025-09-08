use crate::{
    actions::{self, Action},
    builders::lexicon_builder::Lexicon,
    config::Config,
    error::AppError,
    input::{Input, InputContext},
    log_info, theme,
    tracker::Tracker,
    tui,
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{prelude::Backend, Terminal};
use std::time::Duration;

pub struct App {
    pub config: Config,
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
            tracker,
            lexicon,
            should_quit: false,
        }
    }

    pub fn quit(&mut self) -> Result<(), AppError> {
        self.should_quit = true;
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
        if self.tracker.is_idle() {
            self.tracker.start_typing();
        }
        self.tracker.type_char(chr)
    }

    pub fn handle_backspace(&mut self) -> Result<(), AppError> {
        match self.tracker.backspace() {
            Ok(()) => Ok(()),
            Err(AppError::TypingTestNotInProgress) => Ok(()),
            Err(e) => Err(e),
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
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    // TODO: resolve input contxt
                    let action = input.handle(event, InputContext::Typing);
                    if action == Action::Quit {
                        break;
                    }
                    actions::handle_action(&mut app, action)?;
                }
                _ => {}
            }
        }

        terminal.draw(|frame| {
            // TODO: return the click actions
            let _ = tui::renderer::draw_ui(frame, &app);
        })?;
    }

    Ok(())
}
