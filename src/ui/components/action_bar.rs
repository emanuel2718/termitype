use crate::{
    actions::TermiClickAction,
    config::{Mode, ModeType},
    constants::{DEFAULT_TIME_DURATION_LIST, DEFAULT_WORD_COUNT_LIST},
    modal::ModalContext,
    termi::Termi,
    ui::helpers::TermiUtils,
};

use super::elements::TermiElement;

pub struct ActionBarComponent;

impl ActionBarComponent {
    pub fn create(termi: &Termi) -> Vec<TermiElement> {
        let theme = termi.current_theme();
        let config = &termi.config;
        let current_value = config.current_mode().value();
        let is_time_mode = matches!(config.current_mode(), Mode::Time { .. });

        let presets = if is_time_mode {
            DEFAULT_TIME_DURATION_LIST
        } else {
            DEFAULT_WORD_COUNT_LIST
        };

        let is_custom_active = !presets.contains(&current_value);
        let custom_ctx = if is_time_mode {
            ModalContext::CustomTime
        } else {
            ModalContext::CustomWordCount
        };

        let symbols = TermiUtils::get_symbols(theme.supports_unicode());

        let mut elements = vec![
            // toggles
            TermiElement::new(
                format!("{} punctuation ", symbols.punctuation),
                config.use_punctuation,
                Some(TermiClickAction::TogglePunctuation),
            ),
            TermiElement::new(
                format!("{} numbers ", symbols.numbers),
                config.use_numbers,
                Some(TermiClickAction::ToggleNumbers),
            ),
            TermiElement::new(
                format!("{} symbols ", symbols.symbols),
                config.use_symbols,
                Some(TermiClickAction::ToggleSymbols),
            ),
            // spacers
            TermiElement::spacer(1),
            TermiElement::text(symbols.divider),
            TermiElement::spacer(1),
            // modes types
            TermiElement::new(
                "T time ",
                is_time_mode,
                Some(TermiClickAction::SwitchMode(ModeType::Time)),
            ),
            TermiElement::new(
                "A words ",
                !is_time_mode,
                Some(TermiClickAction::SwitchMode(ModeType::Words)),
            ),
            // spacers
            TermiElement::spacer(1),
            TermiElement::text(symbols.divider),
            TermiElement::spacer(1),
        ];

        // modes values
        for &preset in &presets {
            elements.push(TermiElement::new(
                format!("{} ", preset),
                current_value == preset,
                Some(TermiClickAction::SetModeValue(preset)),
            ));
        }

        // custom mode value
        elements.push(TermiElement::new(
            format!("{} ", symbols.custom),
            is_custom_active,
            Some(TermiClickAction::ToggleModal(custom_ctx)),
        ));

        // styling
        elements
            .into_iter()
            .map(|element| element.to_styled(theme))
            .collect()
    }
}
