use crate::config::ModeType;

#[derive(Debug, Clone, PartialEq)]
pub enum TermiClickAction {
    SwitchMode(ModeType),
    SetModeValue(usize),
    TogglePunctuation,
    ToggleSymbols,
    ToggleNumbers,
    ToggleThemePicker,
    ToggleLanguagePicker,
    ToggleAbout,
}
