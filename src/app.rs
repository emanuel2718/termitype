use crate::{
    actions::{self, Action},
    config::Config,
    input::{Input, InputContext},
    log_info, theme,
};
use crossterm::event::{self, Event, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{prelude::Backend, style::Style, widgets::Paragraph, Terminal};
use std::time::Duration;

pub struct App {
    pub config: Config,
    should_quit: bool,
}

impl App {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            should_quit: false,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> anyhow::Result<()> {
    let mut input = Input::new();
    let mut app = App::new(config);

    theme::init_from_config(config)?;

    log_info!("The config: {config:?}");
    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    // TODO: resolve input contxt
                    let action = input.handle(event, InputContext::Typing);
                    actions::handle_action(&mut app, action)?;
                    if app.should_quit {
                        break;
                    }
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    ..
                }) => {
                    break;
                }
                _ => {}
            }
        }

        terminal.draw(|frame| {
            let area = frame.area();
            let current_theme = theme::current_theme();
            let bg = current_theme.bg();
            let fg = current_theme.fg();
            frame.render_widget(
                Paragraph::new("ctrl-c to quit").style(Style::default().bg(bg).fg(fg)),
                area,
            );
        })?;
    }

    Ok(())
}
