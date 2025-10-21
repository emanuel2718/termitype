use crate::{app::App, theme::Theme, variants::PickerVariant};
use ratatui::{Frame, layout::Rect};

pub mod search_bar;
pub mod telescope;
pub mod visualizer;

pub struct Picker;

impl Picker {
    pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
        const MAX_MENU_HEIGHT: u16 = 20;
        let variant = app.config.current_picker_variant();
        let max_height = MAX_MENU_HEIGHT.min(area.height.saturating_sub(6));
        let menu_height = max_height.saturating_sub(2); // borders
        app.menu.ui_height = menu_height as usize;

        match variant {
            PickerVariant::Telescope => telescope::render_telescope_picker(frame, app, theme, area),
            _ => telescope::render_telescope_picker(frame, app, theme, area),
        }
    }
}
