use crate::constants::{
    MAX_CUSTOM_LINE_COUNT, MAX_CUSTOM_TIME, MAX_CUSTOM_WORD_COUNT, MIN_CUSTOM_TIME,
    MIN_CUSTOM_WORD_COUNT,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Modal {
    pub ctx: ModalContext,
    pub kind: ModalKind,
    pub title: String,
    pub description: String,
    pub buffer: Option<InputBuffer>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalKind {
    Input,
    Confirmation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalContext {
    CustomTime,
    CustomWordCount,
    CustomLineCount,
    ExitConfirmation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputBuffer {
    pub input: String,
    pub cursor_pos: usize,
    pub is_numeric: bool,
    pub error: Option<String>,
    pub min_val: u16,
    pub max_val: u16,
}

impl Modal {
    pub fn new(ctx: ModalContext) -> Self {
        match ctx {
            ModalContext::CustomTime => Modal {
                ctx,
                kind: ModalKind::Input,
                title: "Custom Time".to_string(),
                description: "Enter desired test duration (seconds)".to_string(),
                buffer: Some(InputBuffer {
                    input: String::new(),
                    cursor_pos: 0,
                    is_numeric: true,
                    error: None,
                    min_val: MIN_CUSTOM_TIME as u16,
                    max_val: MAX_CUSTOM_TIME as u16,
                }),
            },
            ModalContext::CustomWordCount => Modal {
                ctx,
                kind: ModalKind::Input,
                title: "Custom Word Count".to_string(),
                description: "Enter desired word count".to_string(),
                buffer: Some(InputBuffer {
                    input: String::new(),
                    cursor_pos: 0,
                    is_numeric: true,
                    error: None,
                    min_val: MIN_CUSTOM_WORD_COUNT as u16,
                    max_val: MAX_CUSTOM_WORD_COUNT as u16,
                }),
            },
            ModalContext::CustomLineCount => Modal {
                ctx,
                kind: ModalKind::Input,
                title: "Custom Line Count".to_string(),
                description: "How many visible lines?".to_string(),
                buffer: Some(InputBuffer {
                    input: String::new(),
                    cursor_pos: 0,
                    is_numeric: true,
                    error: None,
                    min_val: 1,
                    max_val: MAX_CUSTOM_LINE_COUNT as u16,
                }),
            },
            ModalContext::ExitConfirmation => Modal {
                ctx,
                kind: ModalKind::Confirmation,
                title: "Exit Termitype".to_string(),
                description: "Are you sure you want to exit?".to_string(),
                buffer: None,
            },
        }
    }

    pub fn handle_input(&mut self, c: char) {
        if let Some(buf) = self.buffer.as_mut() {
            let is_valid_char = if buf.is_numeric {
                c.is_ascii_digit()
            } else {
                c.is_alphabetic() || c.is_whitespace()
            };

            if is_valid_char {
                buf.input.insert(buf.cursor_pos, c);
                buf.cursor_pos += 1;
                Self::validate_input(buf);
                if buf.error.is_some() {
                    buf.cursor_pos -= 1;
                    buf.input.remove(buf.cursor_pos);
                }
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        if let Some(buf) = self.buffer.as_mut() {
            buf.error = None;
            if buf.cursor_pos > 0 {
                buf.cursor_pos -= 1;
                buf.input.remove(buf.cursor_pos);
                Self::validate_input(buf);
            }
        }
    }

    pub fn get_value(&self) -> Result<String, &str> {
        // TODO: add custom error
        if let Some(buf) = &self.buffer {
            if buf.error.is_some() || buf.input.is_empty() {
                return Err("Invalid Input");
            } else {
                return Ok(buf.input.clone());
            }
        }
        Err("No buffer present, he went for milk...a while ago")
    }

    pub fn is_confirmation_modal(&self) -> bool {
        matches!(self.kind, ModalKind::Confirmation)
    }

    fn validate_input(buf: &mut InputBuffer) {
        if buf.input.is_empty() {
            buf.error = Some("Input field cannot be empty".to_string());
            return;
        }

        if buf.is_numeric {
            match buf.input.parse::<u64>() {
                Ok(val) => {
                    if val < buf.min_val as u64 {
                        buf.error = Some(format!("Value must be at least {}", buf.min_val))
                    } else if val > buf.max_val as u64 {
                        buf.error = Some(format!("Value must not exceed {}", buf.max_val))
                    } else {
                        buf.error = None
                    }
                }
                Err(_) => buf.error = Some("Invalid number format".to_string()),
            }
        } else {
            let len = buf.input.len();
            if len < buf.min_val as usize {
                buf.error = Some(format!("Input must be at least {} chars", buf.min_val))
            } else if len > buf.max_val as usize {
                buf.error = Some(format!("Input must not exceed {} chars", buf.max_val))
            } else {
                buf.error = None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_creation() {
        let modal = Modal::new(ModalContext::CustomTime);
        assert_eq!(modal.title, "Custom Time");
        assert_eq!(modal.description, "Enter desired test duration (seconds)");
        assert_eq!(modal.kind, ModalKind::Input);
        assert!(modal.buffer.is_some());
        assert!(modal.buffer.as_ref().unwrap().input.is_empty());
        assert!(modal.buffer.as_ref().unwrap().error.is_none());
        assert_eq!(
            modal.buffer.as_ref().unwrap().min_val,
            MIN_CUSTOM_TIME as u16
        );
        assert_eq!(modal.buffer.unwrap().max_val, MAX_CUSTOM_TIME as u16);

        let confirmation_modal = Modal::new(ModalContext::ExitConfirmation);
        assert_eq!(confirmation_modal.kind, ModalKind::Confirmation);
        assert!(confirmation_modal.buffer.is_none());
    }

    #[test]
    fn test_modal_valid_input() {
        let mut modal = Modal::new(ModalContext::CustomTime);
        modal.handle_input('6');
        modal.handle_input('0');

        assert_eq!(modal.get_value(), Ok("60".to_string()));
        assert!(modal.buffer.is_some());
        assert!(modal.buffer.unwrap().error.is_none());
    }

    #[test]
    fn test_modal_invalid_input() {
        let mut modal = Modal::new(ModalContext::CustomTime);
        modal.handle_input('0');
        assert!(modal.buffer.as_ref().unwrap().error.is_some());

        assert_eq!(modal.get_value(), Err("Invalid Input"));
    }

    #[test]
    fn test_modal_backsapce() {
        let mut modal = Modal::new(ModalContext::CustomTime);
        modal.handle_input('1');
        modal.handle_input('2');
        modal.handle_input('3');
        assert_eq!(modal.get_value(), Ok("123".to_string()));

        modal.handle_backspace();
        assert_eq!(modal.get_value(), Ok("12".to_string()));

        modal.handle_backspace();
        assert_eq!(modal.get_value(), Ok("1".to_string()));

        modal.handle_backspace();
        assert!(modal.buffer.as_ref().unwrap().input.is_empty());
    }

    #[test]
    fn test_modal_backsapce_edge() {
        let mut modal = Modal::new(ModalContext::CustomWordCount);
        let numbers = "5000";
        assert_eq!(numbers, MAX_CUSTOM_WORD_COUNT.to_string());
        for c in numbers.chars() {
            modal.handle_input(c);
        }
        modal.handle_input('1'); // should add nothing, because we are at the limit currently
        assert_eq!(modal.get_value(), Err("Invalid Input"));
        modal.handle_backspace();
        assert_eq!(modal.get_value(), Ok("500".to_string()));
        modal.handle_backspace();
        assert_eq!(modal.get_value(), Ok("50".to_string()));
    }

    #[test]
    fn test_input_error_and_then_correct_input() {
        let mut modal = Modal::new(ModalContext::CustomLineCount);
        modal.handle_input('1');
        modal.handle_input('2');
        modal.handle_input('0');
        assert_eq!(modal.get_value(), Ok("10".to_string()));
    }

    #[test]
    fn test_modal_above_max_value() {
        let mut modal = Modal::new(ModalContext::CustomTime);
        modal.handle_input('3');
        modal.handle_input('0');
        modal.handle_input('1');
        assert_eq!(
            modal.buffer.as_ref().unwrap().error,
            Some("Value must not exceed 300".to_string())
        );
        assert_eq!(modal.get_value(), Err("Invalid Input"));
    }

    #[test]
    fn test_modal_empty_after_backspace() {
        let mut modal = Modal::new(ModalContext::CustomTime);
        modal.handle_input('1');
        assert_eq!(modal.get_value(), Ok("1".to_string()));
        modal.handle_backspace();
        assert_eq!(
            modal.buffer.as_ref().unwrap().error,
            Some("Input field cannot be empty".to_string())
        );
        assert_eq!(modal.get_value(), Err("Invalid Input"));
    }
}
