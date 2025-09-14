use crate::{
    app::App,
    theme::{self, Theme},
    tui::{
        elements::{
            create_command_area, create_footer_element, create_mode_line, create_title,
            create_typing_area,
        },
        layout::{create_main_layout, create_results_layout, AppLayout, ResultsLayout},
        results::{create_minimal_results_display, create_results_footer_element},
    },
    variants::ResultsVariant,
};
use anyhow::Result;
use ratatui::{style::Style, widgets::Block, Frame};

pub fn draw_ui(frame: &mut Frame, app: &mut App) -> Result<()> {
    let area = frame.area();
    let theme = theme::current_theme();

    // outer background
    let bg_block = Block::default().style(Style::default().bg(theme.bg()));
    frame.render_widget(bg_block, area);

    if app.tracker.is_idle() {
        let layout: AppLayout = create_main_layout(area);
        render_idle_screen(frame, app, &theme, layout);
    } else if app.tracker.is_typing() {
        let layout: AppLayout = create_main_layout(area);
        render_typing_screen(frame, app, &theme, layout);
    } else if app.tracker.is_complete() {
        let results_layout = create_results_layout(area);
        render_results_screen(frame, app, &theme, results_layout);
    }
    //
    // // overlays
    // if app.menu.is_open() {
    //     render_menu(frame, app, area);
    // }
    Ok(())
}
/// Render the idle screen. This renders when the user is not typing or actively seeing the Results screen
/// The <lang>, <cmd_bar> and <footer> are known as the `extra` sections.
/// What the above means is that if the screen size is small enough we hide those sections first.
///
/// The rough IDLE screen will look something like this:
///
///  ------------------------
/// |  <title>               |
/// |                        |
/// |                        |
/// |         <lang>         |
/// |      <typing_area>     |
/// |                        |
/// |        <cmd_bar>       |
/// |               <footer> |
///  ------------------------
///
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

/// Render the typing screen. This only renders when actively typing (`TypingStatus::InProgress`)
/// The rough screen will look something like this:
///
///  ------------------------
/// |  <title>               |
/// |                        |
/// |                        |
/// |     <mode_bar>         |
/// |     <typing_area>      |
/// |                        |
/// |                        |
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

/// Render the results screen. This render when the typing test is completed (`TypingStatus::Completed`)
/// The way this looks varies depending on the current `ResultsVariant`.
fn render_results_screen(frame: &mut Frame, app: &mut App, theme: &Theme, layout: ResultsLayout) {
    let results_display = match app.config.current_results_variant() {
        ResultsVariant::Minimal => {
            create_minimal_results_display(
                app,
                theme,
                layout.results_area.height,
                layout.results_area.width,
            )
        }
        // TODO: implement Graph, Neofetch variants later
        ResultsVariant::Graph => todo!(),
        ResultsVariant::Neofetch => todo!(),
    };
    frame.render_widget(results_display, layout.results_area);

    // footer
    let footer_element = create_results_footer_element(theme);
    frame.render_widget(footer_element, layout.footer_area);
}
