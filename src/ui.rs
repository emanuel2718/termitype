use ratatui::{layout::{Alignment, Constraint, Direction, Layout}, prelude::Backend, widgets::{Block, Borders, Paragraph}, Frame};

use crate::termi::Termi;

pub fn draw_ui(f: &mut Frame, termi: &Termi) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
            Constraint::Percentage(10)
        ]).split(f.area());

    let text = Paragraph::new(&*termi.target_text).block(Block::default().borders(Borders::ALL).title("Termitype"));
    f.render_widget(text, chunks[0]);

    let input = Paragraph::new(&*termi.input)
        .block(Block::default().borders(Borders::ALL).title("Your input"));
    f.render_widget(input, chunks[1]);


    if termi.is_finished {
     let finished_message = Paragraph::new("Test Complete! Press Enter to restart.")
            .block(Block::default().borders(Borders::ALL).title("Finished"))
            .alignment(Alignment::Center);
        f.render_widget(finished_message, f.area());
    }
}
