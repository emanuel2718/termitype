use crate::{app::App, theme::Theme, tui::layout::ResultsLayout, variants::ResultsVariant};
use ratatui::Frame;

pub mod bottom_bar;
pub mod graph;
pub mod minimal;
pub mod neofetch;

pub struct Results;

impl Results {
    pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme, layout: ResultsLayout) {
        let current_variant = app.config.current_results_variant();
        let area = layout.results_area;

        match current_variant {
            ResultsVariant::Minimal => minimal::render(frame, app, theme, area),
            ResultsVariant::Graph => graph::render(frame, app, theme, area),
            ResultsVariant::Neofetch => neofetch::render(frame, app, theme, area),
        }

        bottom_bar::render_bar(frame, theme, current_variant, layout.footer_area);
    }
}
