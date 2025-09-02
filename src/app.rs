use crate::{
    actions::Action,
    config::Config,
    input::{Input, InputContext},
    log_info,
};
use crossterm::event::{self, Event, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{prelude::Backend, widgets::Paragraph, Terminal};
use std::time::Duration;

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> anyhow::Result<()> {
    let mut input = Input::new();
    log_info!("The config: {config:?}");
    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    // TODO: resolve input contxt
                    let action = input.handle(event, InputContext::Typing);
                    if action == Action::Quit {
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
            frame.render_widget(Paragraph::new("Click to quit"), area);
        })?;
    }

    Ok(())
}
