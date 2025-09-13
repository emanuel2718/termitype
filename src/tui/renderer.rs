use crate::{
    app::App,
    theme::{self, Theme},
    tui::{
        elements::{
            create_command_area, create_footer_element, create_mode_line, create_title,
            create_typing_area,
        },
        layout::{create_layout, AppLayout},
    },
};
use anyhow::Result;
use ratatui::{style::Style, widgets::Block, Frame};

pub fn draw_ui(frame: &mut Frame, app: &mut App) -> Result<()> {
    let area = frame.area();
    let theme = theme::current_theme();

    let layout: AppLayout = create_layout(area);

    // outer background
    let bg_block = Block::default().style(Style::default().bg(theme.bg()));
    frame.render_widget(bg_block, area);

    if app.tracker.is_idle() {
        render_idle_screen(frame, app, &theme, layout);
    } else if app.tracker.is_typing() {
        render_typing_screen(frame, app, &theme, layout);
    } else if app.tracker.is_complete() {
        render_results_screen(frame, app, &theme, layout);
    }
    //
    // // overlays
    // if app.menu.is_open() {
    //     render_menu(frame, app, area);
    // }
    Ok(())
}

/// Render the typing screen. This only renders when actively typing (`TypingStatus::InProgress`)
/// The rough screen will look something like this:
///
///  ------------------------
/// |  <title>               |
/// |                        |
/// |     <mode_bar>         |
/// |     <typing_area>      |
/// |                        |
///  ------------------------
///
fn render_typing_screen(frame: &mut Frame, app: &mut App, theme: &Theme, layout: AppLayout) {
    // title
    let title = create_title(app, theme);
    frame.render_widget(title, layout.top_area);

    // typing area
    let typing_area = create_typing_area(frame, app, theme, &layout);
    frame.render_widget(typing_area, layout.center_area);
}

fn render_idle_screen(frame: &mut Frame, app: &mut App, theme: &Theme, layout: AppLayout) {
    // title
    let title = create_title(app, theme);
    frame.render_widget(title, layout.top_area);

    // mode line
    let mode_line = create_mode_line(app, theme);
    frame.render_widget(mode_line, layout.top_area);

    // typing area
    let typing_area = create_typing_area(frame, app, theme, &layout);
    frame.render_widget(typing_area, layout.center_area);

    // commands
    let commands_area = create_command_area(theme);
    frame.render_widget(commands_area, layout.command_area);

    // footer
    let footer_element = create_footer_element(theme);
    frame.render_widget(footer_element, layout.footer_area);
}

fn render_results_screen(frame: &mut Frame, app: &App, theme: &Theme, layout: AppLayout) {}
