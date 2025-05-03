use crate::constants::{
    MAX_CUSTOM_TIME, MAX_CUSTOM_WORD_COUNT, MIN_CUSTOM_TIME, MIN_CUSTOM_WORD_COUNT,
};

/// Used to determine which content to show on the modal
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModalContext {
    CustomTime,
    CustomWordCount,
}

#[derive(Debug, Clone)]
pub struct InputBuffer {
    input: String,
    cursor_pos: usize,
    is_numeric: bool,
    error_msg: Option<String>,
    min_value: u16,
    max_value: u16,
}

#[derive(Debug, Clone)]
pub struct InputModal {
    pub ctx: ModalContext,
    pub title: String,
    pub description: String,
    pub buffer: InputBuffer,
}

impl Default for InputModal {
    fn default() -> Self {
        Self {
            ctx: ModalContext::CustomTime,
            title: "<title>".to_string(),
            description: "<description>".to_string(),
            buffer: InputBuffer {
                input: String::new(),
                is_numeric: true,
                cursor_pos: 0,
                error_msg: None,
                min_value: 1,
                max_value: 5,
            },
        }
    }
}

impl InputModal {
    pub fn new(
        ctx: ModalContext,
        title: String,
        description: String,
        is_numeric: bool,
        min: u16,
        max: u16,
    ) -> Self {
        Self {
            ctx,
            title,
            description,
            buffer: InputBuffer {
                input: String::new(),
                is_numeric,
                cursor_pos: 0,
                error_msg: None,
                min_value: min,
                max_value: max,
            },
        }
    }

    pub fn handle_char(&mut self, c: char) {
        let is_numeric = self.buffer.is_numeric;
        if (is_numeric && c.is_ascii_digit()) || (!is_numeric && c.is_alphabetic()) {
            println!("Registering char: {c}");
            self.buffer.input.insert(self.buffer.cursor_pos, c);
            self.buffer.cursor_pos += 1;
            self.validate_input();
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.buffer.cursor_pos > 0 {
            self.buffer.cursor_pos -= 1;
            self.buffer.input.remove(self.buffer.cursor_pos);
            self.validate_input();
        }
    }

    fn validate_input(&mut self) {
        if self.buffer.input.is_empty() {
            self.buffer.error_msg = Some("Input field cannot be empty".to_string());
            return;
        }

        if self.buffer.is_numeric {
            match self.buffer.input.parse::<u64>() {
                Ok(value) => {
                    let min_msg =
                        Some(format!("Value must be at least: {}", self.buffer.min_value));
                    let max_msg = Some(format!("Value must not exceed: {}", self.buffer.max_value));
                    if value < self.buffer.min_value as u64 {
                        self.buffer.error_msg = min_msg;
                    } else if value > self.buffer.max_value as u64 {
                        self.buffer.error_msg = max_msg;
                    } else {
                        self.buffer.error_msg = None;
                    }
                }
                Err(_) => {
                    self.buffer.error_msg = Some("Invalid number format.".to_string());
                }
            }
        } else {
            let input_len = self.buffer.input.len();
            let min_len = self.buffer.min_value as usize;
            let max_len = self.buffer.max_value as usize;
            let min_msg = Some(format!("Input must be at least {}", min_len));
            let max_msg = Some(format!("Input must not exceed {}", max_len));
            if input_len < min_len {
                self.buffer.error_msg = min_msg;
            } else if input_len > max_len {
                self.buffer.error_msg = max_msg;
            } else {
                self.buffer.error_msg = None;
            }
        }
    }
}
pub fn build_modal(ctx: ModalContext) -> InputModal {
    match ctx {
        ModalContext::CustomTime => InputModal {
            ctx,
            title: "Custom Time".to_string(),
            description: "Enter desired test duration (seconds)".to_string(),
            buffer: InputBuffer {
                input: "".to_string(),
                cursor_pos: 0,
                is_numeric: true,
                error_msg: None,
                min_value: MIN_CUSTOM_TIME,
                max_value: MAX_CUSTOM_TIME,
            },
        },
        ModalContext::CustomWordCount => InputModal {
            ctx,
            title: "Custom Word Count".to_string(),
            description: "Enter desired word count".to_string(),
            buffer: InputBuffer {
                input: "".to_string(),
                cursor_pos: 0,
                is_numeric: true,
                error_msg: None,
                min_value: MIN_CUSTOM_WORD_COUNT,
                max_value: MAX_CUSTOM_WORD_COUNT,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_custom_modal(is_numeric: bool, min: u16, max: u16) -> InputModal {
        InputModal {
            ctx: ModalContext::CustomTime,
            title: "Test Modal".to_string(),
            description: "Random Test Modal".to_string(),
            buffer: InputBuffer {
                input: String::new(),
                cursor_pos: 0,
                is_numeric,
                error_msg: None,
                min_value: min,
                max_value: max,
            },
        }
    }

    #[test]
    fn test_modal_creation() {
        let modal = InputModal::default();
        assert_eq!(modal.title, "<title>".to_string());
        assert_eq!(modal.description, "<description>".to_string());
        assert!(modal.buffer.input.is_empty());
        assert!(modal.buffer.error_msg.is_none());
    }

    #[test]
    fn test_numeric_input() {
        let is_numeric = true;
        let mut modal = create_custom_modal(is_numeric, 1, 3);
        assert!(modal.buffer.input.is_empty());

        assert!(modal.buffer.error_msg.is_none());
        modal.handle_char('e');
        assert!(modal.buffer.input.is_empty());

        modal.handle_char('1');
        assert!(modal.buffer.input.len() == 1);
        assert!(modal.buffer.error_msg.is_none());
        modal.handle_char('0'); // now the input should be 10
        assert!(modal.buffer.error_msg.is_some());
    }

    #[test]
    fn test_char_input() {
        let is_numeric = false;
        let mut modal = create_custom_modal(is_numeric, 1, 2);
        assert!(modal.buffer.input.is_empty());

        assert!(modal.buffer.error_msg.is_none());
        modal.handle_char('1');
        assert!(modal.buffer.input.is_empty());

        modal.handle_char('e');
        assert!(modal.buffer.input.len() == 1);
        assert!(modal.buffer.error_msg.is_none());
        modal.handle_char('e'); // now the input should be `ee`
        assert!(modal.buffer.error_msg.is_none());
        modal.handle_char('e'); // now the input should be `eee` exceeding max
        assert!(modal.buffer.error_msg.is_some());
    }
    #[test]
    fn test_backspace() {
        let is_numeric = false;
        let mut modal = create_custom_modal(is_numeric, 1, 1);
        assert!(modal.buffer.input.is_empty());

        modal.handle_char('e');
        assert!(modal.buffer.input.len() == 1);
        modal.handle_backspace();
        assert!(modal.buffer.input.is_empty());

        modal.handle_char('e');
        modal.handle_char('e');
        assert!(modal.buffer.error_msg.is_some()); // exceeds current max of `1`
        modal.handle_backspace();
        assert!(modal.buffer.error_msg.is_none());
    }
}
