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

        let height = layout.results_area.height;
        let width = layout.results_area.width;

        let screen = match current_variant {
            ResultsVariant::Minimal => minimal::create_minimal_results(app, theme, height, width),
            ResultsVariant::Graph => graph::create_graph_results(theme, height, width),
            ResultsVariant::Neofetch => neofetch::create_neofetch_results(theme, height, width),
        };

        frame.render_widget(screen, layout.results_area);

        bottom_bar::render_bar(frame, theme, current_variant, layout.footer_area);
    }
}
