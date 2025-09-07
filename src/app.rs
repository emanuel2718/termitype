use crate::{
    actions::{self, Action},
    builders::lexicon_builder::Lexicon,
    config::Config,
    frontend,
    input::{Input, InputContext},
    log_info, theme,
};
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{prelude::Backend, Terminal};
use std::time::Duration;

pub struct App {
    pub config: Config,
    pub lexicon: Lexicon,
    should_quit: bool,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let lexicon = Lexicon::new(config).unwrap();

        Self {
            config: config.clone(),
            lexicon,
            should_quit: false,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn start(&mut self) {
        // NOTE: if we start a new test we want to clear the custom words flag as starting a new
        //       test is designed to generate a completely new test. If the user want to keep
        //       the custom words then a `Redo` is the option.
        if self.config.cli.words.is_some() {
            self.config.cli.clear_custom_words_flag();
        }
        self.lexicon.regenerate(&self.config).unwrap();
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
            let _ = frontend::renderer::draw_ui(frame, &app);
        })?;
    }

    Ok(())
}
