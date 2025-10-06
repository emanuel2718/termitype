use crate::{
    app::App,
    theme::{self, Theme},
    tui::{
        components::{
            command_bar, footer,
            modal_dialog::ModalDialog,
            mode_bar, pickers,
            results::{create_minimal_results_display, create_results_footer_element},
            size_warning, title, typing_area,
        },
        layout::{
            create_main_layout, create_results_layout, AppLayout, LayoutBuilder, ResultsLayout,
        },
    },
    variants::{PickerVariant, ResultsVariant},
};
use anyhow::Result;
use ratatui::{layout::Rect, style::Style, widgets::Block, Frame};

pub fn draw_ui(frame: &mut Frame, app: &mut App) -> Result<()> {
    let area = frame.area();
    let theme = theme::current_theme();

    // outer background
    let bg_block = Block::default().style(Style::default().bg(theme.bg()));
    frame.render_widget(bg_block, area);

    // that's what she said
    if LayoutBuilder::is_too_smol(area) {
        if area.height >= 2 && area.width >= 1 {
            let (warning, warning_width) =
                size_warning::create_size_warning_element(&theme, area.height, area.width);
            let warning_height = 2;
            let clamped_width = warning_width.min(area.width);
            let x = ((area.width as i32 - clamped_width as i32).max(0) / 2) as u16;
            let y = ((area.height as i32 - warning_height as i32).max(0) / 2) as u16;
            let width = clamped_width;
            let height = warning_height;
            let warning_rect = Rect::new(x, y, width, height);
            frame.render_widget(warning, warning_rect);
        }
        return Ok(());
    }

    match app.tracker.is_complete() {
        true => render_results_screen(frame, app, &theme, create_results_layout(area)),
        false => {
            let layout = create_main_layout(area);
            match app.tracker.is_idle() {
                true => render_idle_screen(frame, app, &theme, layout),
                false => render_typing_screen(frame, app, &theme, layout),
            }
        }
    }
    // TODO: have a flag like `app.an_overlay_open` or something like that
    try_render_overlays(frame, app, &theme, area);

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
/// |        <mode>          |
/// |                        |
/// |        <lang>          |
/// |     <typing_area>      |
/// |                        |
/// |       <cmd_bar>        |
/// |               <footer> |
///  ------------------------
///
fn render_idle_screen(frame: &mut Frame, app: &mut App, theme: &Theme, layout: AppLayout) {
    // title
    if let Some(rect) = layout.title_area {
        let title = title::create_title(app, theme);
        frame.render_widget(title, rect);
    }

    // action bar
    if let Some(rect) = layout.mode_bar_area {
        let mode_line = mode_bar::create_mode_line(app, theme, rect.height, rect.width);
        frame.render_widget(mode_line, rect);
    }

    // typing area
    typing_area::render_typing_area(frame, app, theme, &layout);

    // commands
    if layout.show_command_bar {
        let commands_area = command_bar::create_command_area(theme);
        frame.render_widget(commands_area, layout.command_area);
    }

    // footer
    if layout.show_footer {
        let footer_element = footer::create_footer_element(theme);
        frame.render_widget(footer_element, layout.footer_area);
    }
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
    if let Some(rect) = layout.title_area {
        let title = title::create_title(app, theme);
        frame.render_widget(title, rect);
    }

    // typing area
    typing_area::render_typing_area(frame, app, theme, &layout);
}

/// Render the results screen. This render when the typing test is completed (`TypingStatus::Completed`)
/// The way this looks varies depending on the current `ResultsVariant`.
fn render_results_screen(frame: &mut Frame, app: &mut App, theme: &Theme, layout: ResultsLayout) {
    // TODO: use this pattern
    // Results::render(...);
    let results_display = match app.config.current_results_variant() {
        ResultsVariant::Minimal => create_minimal_results_display(
            app,
            theme,
            layout.results_area.height,
            layout.results_area.width,
        ),
        // TODO: implement Graph, Neofetch variants later
        ResultsVariant::Graph => todo!(),
        ResultsVariant::Neofetch => todo!(),
    };
    frame.render_widget(results_display, layout.results_area);

    // footer
    let footer_element = create_results_footer_element(theme);
    frame.render_widget(footer_element, layout.footer_area);
}

fn try_render_overlays(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    if let Some(ref modal) = app.modal {
        ModalDialog::render(frame, modal, theme, area);
    } else if app.menu.is_open() {
        // TODO: use this pattern
        // Picker::render(frame, app, theme, area);
        render_menu_picker(frame, app, theme, area);
    }
}

fn render_menu_picker(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    const MAX_MENU_HEIGHT: u16 = 20;
    let variant = app.config.current_picker_variant();
    let max_height = MAX_MENU_HEIGHT.min(area.height.saturating_sub(6));
    let menu_height = max_height.saturating_sub(2); // borders
    app.menu.ui_height = menu_height as usize;

    match variant {
        PickerVariant::Telescope => pickers::render_telescope_picker(frame, app, theme, area),
        _ => pickers::render_telescope_picker(frame, app, theme, area),
    }
}
