#[derive(Debug, Clone, Copy)]
pub struct WordPosition {
    pub start_index: usize,
    pub line: usize,
    pub col: usize,
}

pub fn calculate_word_positions(text: &str, available_width: usize) -> Vec<WordPosition> {
    if available_width == 0 {
        return vec![];
    }
    let word_count = text.split_whitespace().count();
    let mut positions = Vec::with_capacity(word_count);
    let mut current_line = 0;
    let mut current_col = 0;
    let mut current_index = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        let word_fits_on_line = word_len < available_width;

        if current_col > 0
            && (current_col + word_len >= available_width
                || (word_fits_on_line && current_col + 1 >= available_width))
        {
            current_line += 1;
            current_col = 0;
        }

        if word_len >= available_width && current_col > 0 {
            current_line += 1;
            current_col = 0;
        }

        positions.push(WordPosition {
            start_index: current_index,
            line: current_line,
            col: current_col,
        });

        let next_col = current_col + word_len + 1;

        current_index += word_len + 1;

        if next_col >= available_width {
            current_line += 1;
            current_col = 0;
        } else {
            current_col = next_col;
        }
    }

    positions
}
