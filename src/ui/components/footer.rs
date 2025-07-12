use ratatui::symbols::line::DOUBLE_VERTICAL_LEFT;

use crate::{
    actions::{MenuContext, TermiClickAction},
    termi::Termi,
    ui::helpers::TermiUtils,
    version::VERSION,
};

use super::elements::TermiElement;

pub struct FooterComponent;

impl FooterComponent {
    pub fn create(termi: &Termi) -> Vec<TermiElement> {
        let theme = termi.current_theme();
        let symbols = TermiUtils::get_symbols(theme.supports_unicode());

        let divider = if theme.supports_unicode() {
            DOUBLE_VERTICAL_LEFT
        } else {
            "|"
        };

        let theme_click_action = if termi.theme.color_support.supports_themes() {
            Some(TermiClickAction::ToggleThemePicker)
        } else {
            None
        };

        let elements = vec![
            TermiElement::new(
                format!("{} about", symbols.info),
                termi.menu.is_current_ctx(MenuContext::About),
                Some(TermiClickAction::ToggleAbout),
            ),
            TermiElement::spacer(1),
            TermiElement::text(divider),
            TermiElement::spacer(1),
            TermiElement::new(
                termi.theme.id.as_ref(),
                termi.preview_theme.is_some(),
                theme_click_action,
            ),
            TermiElement::spacer(1),
            TermiElement::text(divider),
            TermiElement::spacer(1),
            TermiElement::text(format!("v{VERSION}")),
        ];

        elements
            .into_iter()
            .map(|element| element.to_styled(theme))
            .collect()
    }
}
