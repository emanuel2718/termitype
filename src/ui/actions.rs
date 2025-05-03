use crate::{config::ModeType, modal::ModalContext};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TermiClickAction {
    SwitchMode(ModeType),
    SetModeValue(usize),
    ToggleMenu,
    TogglePunctuation,
    ToggleSymbols,
    ToggleNumbers,
    ToggleThemePicker,
    ToggleLanguagePicker,
    ToggleAbout,
    ToggleModal(ModalContext),
    ModalConfirm,
}
