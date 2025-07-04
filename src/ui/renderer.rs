use ratatui::{
    layout::{Alignment, Rect},
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{
    termi::Termi,
    tracker::Status,
    ui::{
        components::{elements::TermiClickableRegions, LeaderboardComponent},
        layout::{create_container_block, TermiLayout},
    },
};

use super::{
    components::{
        ActionBarComponent, CommandBarComponent, FooterComponent, HeaderComponent, MenuComponent,
        ModalComponent, ModeBarComponent, ResultsComponent, SizeWarningComponent, TermiElement,
        TypingAreaComponent,
    },
    helpers::{LayoutHelper, TermiStyle},
    layout::create_layout,
};

/// Main entry point for rendering the stuff in the thing
pub fn draw_ui(frame: &mut Frame, termi: &mut Termi, fps: Option<f64>) -> TermiClickableRegions {
    let mut regions = TermiClickableRegions::default();
    let theme = termi.current_theme();
    let area = frame.area();

    // layout
    let dummy_layout = create_layout(Block::new().inner(area), termi);
    let container = create_container_block(&dummy_layout, theme);
    let inner_area = container.inner(area);

    let layout = if dummy_layout.is_minimal() {
        dummy_layout
    } else {
        create_layout(inner_area, termi)
    };

    // render the outer container
    frame.render_widget(container, area);

    // too small, show the size warning msg
    if layout.is_minimal() {
        let size_warning = SizeWarningComponent::create(termi, inner_area.width, inner_area.height);
        render_termi_elements(frame, &mut regions, size_warning, area);
        return regions;
    }

    // fps counter if currently enabled
    if let Some(fps_value) = fps {
        render_fps_counter(frame, area, fps_value, theme);
    }

    // show different screens depending the current `Tracker::Status`
    match termi.tracker.status {
        Status::Typing => {
            render_typing_screen(frame, &mut regions, termi, &layout);
        }
        Status::Idle | Status::Paused => {
            render_idle_screen(frame, &mut regions, termi, &layout);
        }
        Status::Completed => {
            ResultsComponent::render(frame, termi, area, layout.show_small_results());
        }
    }

    // overlays in this area of town are things like `menu` and `modals`
    render_overlays(frame, &mut regions, termi, area);

    regions
}

/// Render the typing screen. This only renders when actively typing (`Status::Typing`)
/// The rough screen will look something like this:
///
/// -------------------------
/// |  <header>              |
/// |                        |
/// |     <mode_bar>         |
/// |     <typing_area>      |
/// |                        |
/// -------------------------
///
fn render_typing_screen(
    frame: &mut Frame,
    regions: &mut TermiClickableRegions,
    termi: &Termi,
    layout: &super::layout::TermiLayout,
) {
    // title
    let header = HeaderComponent::create(termi);
    render_termi_elements(frame, regions, header, layout.section.header);

    // mode bar
    let mode_bar = ModeBarComponent::create(termi);
    let mode_bar_area = LayoutHelper::center_with_max_width(
        layout.section.mode_bar,
        crate::constants::TYPING_AREA_WIDTH,
    );
    render_termi_elements(frame, regions, mode_bar, mode_bar_area);

    // typing area
    TypingAreaComponent::render(frame, termi, layout.section.typing_area);
}

/// Render the idle screen. This renders when the user is not typing or now seeing Results screen
/// The <lang>, <cmd_bar> and <footer> are known as the `extra` sections.
/// What the above means is that if the screen size is small enough we hide those sections first.
///
/// The rough IDLE screen will look something like this:
///
/// -------------------------
/// |  <header>              |
/// |                        |
/// |         <lang>         |
/// |      <typing_area>     |
/// |                        |
/// |        <cmd_bar>       |
/// |        <footer>        |
/// -------------------------
///
fn render_idle_screen(
    frame: &mut Frame,
    regions: &mut TermiClickableRegions,
    termi: &Termi,
    layout: &super::layout::TermiLayout,
) {
    // title and mode bar
    let header = HeaderComponent::create(termi);
    let mode_bar = ModeBarComponent::create(termi);

    render_termi_elements(frame, regions, header, layout.section.header);
    render_termi_elements(frame, regions, mode_bar, layout.section.mode_bar);

    // typing area
    TypingAreaComponent::render(frame, termi, layout.section.typing_area);

    // extra sections (optionals)
    render_extra_sections(frame, regions, termi, layout)
}

/// Renders the extra sections such as action bar, command bar, and footer sections.
fn render_extra_sections(
    frame: &mut Frame,
    regions: &mut TermiClickableRegions,
    termi: &Termi,
    layout: &TermiLayout,
) {
    // the width is small enough that we cannot render the full action bar
    if layout.w_small() && !layout.h_small() {
        let menu_button = MenuComponent::create_show_menu_button(termi);
        render_termi_elements(frame, regions, menu_button, layout.section.action_bar);

        if layout.show_footer() {
            render_bottom_section(frame, regions, termi, layout);
        }
    // we have enough width and the size is tall enough that we can show the full action bar
    } else if !layout.is_small() {
        let action_bar = ActionBarComponent::create(termi);
        render_termi_elements(frame, regions, action_bar, layout.section.action_bar);

        if layout.show_footer() {
            render_bottom_section(frame, regions, termi, layout);
        }
    }
}

/// Render the bottom section.
/// The bottom sections is comprised of the Footer + Command bar
///
/// -------------------------
/// |                        |
/// |                        |
/// |                        |
/// |                        |
/// |    ----------------    |
/// |   |    <cmd_bar>   |   |
/// |   |    <footer>    |   |
/// |    ----------------    |
/// -------------------------
///
fn render_bottom_section(
    frame: &mut Frame,
    regions: &mut TermiClickableRegions,
    termi: &Termi,
    layout: &super::layout::TermiLayout,
) {
    let command_bar = CommandBarComponent::create(termi);
    let footer = FooterComponent::create(termi);

    render_termi_elements(frame, regions, command_bar, layout.section.command_bar);
    render_termi_elements(frame, regions, footer, layout.section.footer);
}

/// Render overlay components (menu and modal)
fn render_overlays(
    frame: &mut Frame,
    regions: &mut TermiClickableRegions,
    termi: &mut Termi,
    area: Rect,
) {
    if termi.menu.is_open() {
        MenuComponent::render(frame, termi, area);
    }

    if let Some(modal) = &termi.modal {
        if let Some(region) = ModalComponent::render(frame, termi, area, modal.clone()) {
            regions.add(region.0, region.1);
        }
    }

    let leaderboard_is_open = match &termi.leaderboard {
        Some(leaderboard) => leaderboard.is_open(),
        None => false,
    };
    if leaderboard_is_open {
        if let Some(region) = LeaderboardComponent::render(frame, termi, area) {
            regions.add(region.0, region.1);
        }
    }
}

/// Renders TermiElement(s) with proper positioning and clickable region tracking.
///
/// Handles both single and multiple element cases:
/// Each element with an action gets registered as a clickable region.
///
/// **Single Element**
///
/// -------------------------
/// |                        |
/// |    [centered text]     |  <- just one element centered. ez mode
/// |                        |
/// -------------------------
///
/// **Multiple Elements**
///
///  --------------------------
/// |                          |
/// |     [e1] [e2] [e3]       |  <- spans + clickable regions
/// |                          |
///  --------------------------
///
fn render_termi_elements(
    frame: &mut Frame,
    regions: &mut TermiClickableRegions,
    elements: Vec<TermiElement>,
    area: Rect,
) {
    if elements.is_empty() {
        return;
    }

    // single element handling
    if elements.len() == 1 {
        let element = &elements[0];
        let alignment = element.content.alignment.unwrap_or(Alignment::Left);
        let text_height = element.content.height() as u16;
        let text_width = element.content.width() as u16;

        let centered_area = LayoutHelper::center_text_rect(area, &element.content);

        let paragraph = Paragraph::new(element.content.clone()).alignment(alignment);
        frame.render_widget(paragraph, centered_area);

        if let Some(action) = element.action {
            let clickable_area =
                LayoutHelper::clickable_text_area(area, text_width, text_height, alignment);
            regions.add(clickable_area, action);
        }
        return;
    }

    // multiple elmenet handling
    let mut spans = Vec::new();
    let mut total_width: u16 = 0;
    let mut element_data = Vec::new();

    // determines the total width needed
    for element in &elements {
        let line_width = element.content.lines.first().map_or(0, |line| line.width()) as u16;
        total_width += line_width;
        element_data.push((line_width, element.action));
    }

    // determines where the spans are going to sstart
    let start_x = if total_width <= area.width {
        area.x + (area.width.saturating_sub(total_width)) / 2
    } else {
        area.x
    };

    let alignment = if total_width <= area.width {
        Alignment::Center
    } else {
        Alignment::Left
    };

    // spans + clickable regions
    let mut current_x_offset: u16 = 0;
    for (i, element) in elements.iter().enumerate() {
        let (element_width, action) = element_data[i];

        if let Some(line) = element.content.lines.first() {
            spans.extend(line.spans.clone());
        }

        if let Some(action) = action {
            let region_rect = Rect {
                x: start_x + current_x_offset,
                y: area.y,
                width: element_width,
                height: area.height.min(1),
            };
            if element_width > 0 {
                regions.add(region_rect, action);
            }
        }
        current_x_offset += element_width;
    }

    frame.render_widget(Paragraph::new(Line::from(spans)).alignment(alignment), area);
}

/// Render FPS counter.
fn render_fps_counter(frame: &mut Frame, area: Rect, fps: f64, theme: &crate::theme::Theme) {
    let fps_text = format!("FPS: {:.0}", fps);
    let widget = Paragraph::new(fps_text)
        .style(TermiStyle::muted(theme))
        .alignment(Alignment::Right);

    let widget_width = 10;
    let fps_area = Rect::new(
        area.right().saturating_sub(widget_width + 1),
        area.top() + 1,
        widget_width,
        1,
    );

    if fps_area.right() <= frame.area().right() && fps_area.bottom() <= frame.area().bottom() {
        frame.render_widget(widget, fps_area);
    }
}
