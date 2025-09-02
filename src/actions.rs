#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Action {
    NoOp,
    Quit,
    Start,
    Resume,
    Pause,

    Backspace,
}
