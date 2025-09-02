#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Action {
    NoOp,
    Quit,
    Start,
    Resume,
    Redo,
    Pause,

    Input(char),
    Backspace,
}
