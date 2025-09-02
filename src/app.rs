use crate::{config::Config, log_info};
use crossterm::event::{self, Event, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{prelude::Backend, widgets::Paragraph, Terminal};
use std::time::Duration;

pub fn run<B: Backend>(terminal: &mut Terminal<B>, config: &Config) -> anyhow::Result<()> {
    log_info!("The config: {config:?}");
    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    log_info!("event: {event:?}");
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
