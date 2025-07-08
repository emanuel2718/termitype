use ratatui::{
    layout::{Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
    Frame,
};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::{
    actions::MenuContext,
    constants::TYPING_AREA_WIDTH,
    termi::Termi,
    tracker::Status,
    ui::helpers::{calculate_word_positions, LayoutHelper, WordPosition, UI_CACHE},
};

pub struct TypingAreaComponent;

impl TypingAreaComponent {
    pub fn render(frame: &mut Frame, termi: &Termi, area: Rect) {
        let available_width = area.width.min(TYPING_AREA_WIDTH) as usize;
        let line_count = termi.config.visible_lines as usize;
        let cursor_idx = termi.tracker.cursor_position;

        let mut hasher = DefaultHasher::new();
        termi.words.hash(&mut hasher);
        available_width.hash(&mut hasher);
        let cache_key = hasher.finish();

        let word_positions = UI_CACHE.with(|cache| {
            let mut cache_ref = cache.borrow_mut();

            // can we use the cached position question mark
            if cache_ref.last_text_hash == cache_key {
                if let Some(ref positions) = cache_ref.last_word_positions {
                    return positions.clone();
                }
            }

            let positions = calculate_word_positions(&termi.words, available_width);

            cache_ref.last_word_positions = Some(positions.clone());
            cache_ref.last_text_hash = cache_key;
            cache_ref.last_width = available_width;

            positions
        });

        if word_positions.is_empty() {
            frame.render_widget(Paragraph::new(Text::raw("")), area);
            return;
        }

        // cursor position calc
        let current_word_pos_idx =
            match word_positions.binary_search_by(|pos| pos.start_index.cmp(&cursor_idx)) {
                Ok(idx) => idx,
                Err(idx) => idx.saturating_sub(1),
            };

        let current_word_pos = &word_positions[current_word_pos_idx];
        let current_line = current_word_pos.line;

        // scroll calculation logic
        let scroll_offset = LayoutHelper::calculate_scroll_offset(current_line, line_count);

        let typing_text = Self::create_text(termi, scroll_offset, line_count, &word_positions);
        let text_height = typing_text.height();

        let area_width = available_width as u16;
        // let area_padding = (area.width - area_width) >> 1; // better looking than division by 2
        let area_padding = (area.width.saturating_sub(area_width)) / 2;

        let render_area = Rect {
            x: area.x + area_padding,
            y: area.y,
            width: area_width,
            height: area.height,
        };

        let paragraph = Paragraph::new(typing_text).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, render_area);

        let leaderboard_is_open = match &termi.leaderboard {
            Some(leaderboard) => leaderboard.is_open(),
            None => false,
        };

        // cursor rendering logic
        let should_show_cursor = (termi.tracker.status == Status::Idle
            || termi.tracker.status == Status::Typing)
            && termi.modal.is_none()
            && !leaderboard_is_open
            && !termi.menu.is_current_ctx(MenuContext::Theme);

        if should_show_cursor {
            let menu_obscures_cursor = termi.menu.is_open() && {
                let estimated_menu_area =
                    crate::ui::helpers::calculate_menu_area(termi, frame.area());
                estimated_menu_area.intersects(render_area)
            };

            if !menu_obscures_cursor {
                let offset_x = cursor_idx - current_word_pos.start_index;
                let cursor_x = render_area.x + current_word_pos.col as u16 + offset_x as u16;
                let cursor_y = render_area.y + (current_line - scroll_offset) as u16;

                let in_bounds = cursor_x >= render_area.left()
                    && cursor_x < render_area.right()
                    && cursor_y >= render_area.top()
                    && cursor_y < render_area.top() + text_height as u16;

                if in_bounds {
                    frame.set_cursor_position(Position {
                        x: cursor_x,
                        y: cursor_y,
                    });
                }
            }
        }
    }

    pub fn create_text<'a>(
        termi: &'a Termi,
        scroll_offset: usize,
        visible_line_count: usize,
        word_positions: &[WordPosition],
    ) -> Text<'a> {
        let theme = termi.current_theme();

        if word_positions.is_empty() {
            return Text::raw("");
        }

        let words: Vec<&str> = termi.words.split_whitespace().collect();
        let mut lines: Vec<Line> = Vec::with_capacity(visible_line_count);

        let first_line_to_render = scroll_offset;
        let last_line_to_render = scroll_offset + visible_line_count;

        let aprox_spans_per_line = 30; // this is a guesstimate, could be more could be less
        let mut current_line_spans = Vec::with_capacity(aprox_spans_per_line);
        let mut current_line_idx_in_full_text = 0;

        if let Some(first_pos) = word_positions.first() {
            current_line_idx_in_full_text = first_pos.line;
        }

        let cursor_pos = termi.tracker.cursor_position;
        let supports_themes = theme.color_support.supports_themes();

        let success_style = Style::default().fg(theme.success());
        let error_style = Style::default().fg(theme.error());
        let dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);
        let underline_error_style = if supports_themes {
            error_style
                .add_modifier(Modifier::UNDERLINED)
                .underline_color(theme.error())
        } else {
            error_style
        };
        let underline_success_style = if supports_themes {
            success_style
                .add_modifier(Modifier::UNDERLINED)
                .underline_color(theme.error())
        } else {
            success_style
        };

        // skipped characters are ones skipped by the space jumps
        let skipped_style = if supports_themes {
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::UNDERLINED | Modifier::DIM)
                .underline_color(theme.error())
        } else {
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM)
        };

        let mut current_batch = String::with_capacity(64);
        let mut current_batch_style: Option<Style> = None;

        let flush_batch = |batch: &mut String, style: Option<Style>, spans: &mut Vec<Span>| {
            if let Some(s) = style {
                if !batch.is_empty() {
                    spans.push(Span::styled(batch.clone(), s));
                    batch.clear();
                }
            }
        };

        for (idx, word) in words.iter().enumerate() {
            if idx >= word_positions.len() {
                break;
            }

            let word_pos = &word_positions[idx];
            let word_start = word_pos.start_index;
            let word_len = word.chars().count();

            if word_pos.line != current_line_idx_in_full_text {
                // flush any pending batch before changing lines
                flush_batch(
                    &mut current_batch,
                    current_batch_style,
                    &mut current_line_spans,
                );

                if current_line_idx_in_full_text >= first_line_to_render
                    && current_line_idx_in_full_text < last_line_to_render
                {
                    lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                }

                current_line_idx_in_full_text = word_pos.line;
                current_line_spans.clear();
                current_batch_style = None;
            }

            if current_line_idx_in_full_text < first_line_to_render
                || current_line_idx_in_full_text >= last_line_to_render
            {
                continue;
            }

            let is_word_wrong = termi.tracker.is_word_wrong(word_start);
            let is_past_word = cursor_pos > word_start + word_len;
            let should_underline_word = is_word_wrong && is_past_word && supports_themes;

            for (i, c) in word.chars().enumerate() {
                let char_idx = word_start + i;
                let is_correct =
                    termi.tracker.user_input.get(char_idx).copied().flatten() == Some(c);
                let has_input = termi.tracker.user_input.get(char_idx).is_some();

                let is_skipped_by_space_jump =
                    termi.tracker.user_input.get(char_idx) == Some(&None);

                let style = if is_skipped_by_space_jump {
                    skipped_style
                } else if !has_input {
                    dim_style
                } else if is_correct {
                    if should_underline_word {
                        underline_success_style
                    } else {
                        success_style
                    }
                } else if should_underline_word {
                    underline_error_style
                } else {
                    error_style
                };

                if current_batch_style.is_some() && current_batch_style != Some(style) {
                    flush_batch(
                        &mut current_batch,
                        current_batch_style,
                        &mut current_line_spans,
                    );
                }

                current_batch.push(c);
                current_batch_style = Some(style);
            }

            flush_batch(
                &mut current_batch,
                current_batch_style,
                &mut current_line_spans,
            );

            // space in between words handling
            if idx < words.len() - 1 {
                let space_idx = word_start + word.len();
                let space_has_input = termi.tracker.user_input.get(space_idx).is_some();
                let space_is_skipped = termi.tracker.user_input.get(space_idx) == Some(&None);

                let space_style = if space_is_skipped {
                    skipped_style
                } else if space_has_input {
                    success_style
                } else {
                    dim_style
                };

                if current_batch_style.is_some() && current_batch_style != Some(space_style) {
                    flush_batch(
                        &mut current_batch,
                        current_batch_style,
                        &mut current_line_spans,
                    );
                }
                current_batch.push(' ');
                current_batch_style = Some(space_style);
            }
        }
        flush_batch(
            &mut current_batch,
            current_batch_style,
            &mut current_line_spans,
        );

        if current_line_idx_in_full_text >= first_line_to_render
            && current_line_idx_in_full_text < last_line_to_render
        {
            lines.push(Line::from(current_line_spans));
        }

        Text::from(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config, termi::Termi, tracker::Tracker, ui::helpers::calculate_word_positions,
    };
    use ratatui::style::{Modifier, Style};
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_env() -> (TempDir, EnvGuard) {
        let _guard = ENV_MUTEX.lock().unwrap();

        let tmp_dir = TempDir::new().unwrap();
        let tmp_path = tmp_dir.path().to_path_buf();

        let env_guard = if cfg!(target_os = "macos") {
            let original = std::env::var("HOME").ok();
            std::env::set_var("HOME", &tmp_path);
            EnvGuard::new("HOME", original)
        } else if cfg!(target_os = "windows") {
            let original = std::env::var("APPDATA").ok();
            std::env::set_var("APPDATA", &tmp_path);
            EnvGuard::new("APPDATA", original)
        } else {
            let original = std::env::var("XDG_CONFIG_HOME").ok();
            std::env::set_var("XDG_CONFIG_HOME", &tmp_path);
            EnvGuard::new("XDG_CONFIG_HOME", original)
        };

        (tmp_dir, env_guard)
    }

    struct EnvGuard {
        key: &'static str,
        og_val: Option<String>,
    }

    impl EnvGuard {
        fn new(key: &'static str, og_val: Option<String>) -> Self {
            Self { key, og_val }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.og_val {
                Some(val) => std::env::set_var(self.key, val),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn init(target_text: &str) -> (Termi, TempDir, EnvGuard) {
        let (temp_dir, env_guard) = setup_env();
        let config = Config::default();
        let mut termi = Termi::new(&config);
        termi.words = target_text.to_string();
        termi.tracker = Tracker::new(&config, target_text.to_string());
        (termi, temp_dir, env_guard)
    }

    fn simulate_typing(termi: &mut Termi, input: &str) {
        termi.tracker.start_typing();
        for c in input.chars() {
            termi.tracker.type_char(c);
        }
    }

    fn get_all_chars_and_styles(text: &ratatui::text::Text) -> Vec<(char, Style)> {
        let mut result = Vec::new();
        for line in &text.lines {
            for span in &line.spans {
                for ch in span.content.chars() {
                    result.push((ch, span.style));
                }
            }
        }
        result
    }

    #[test]
    fn test_ui_handles_correctly_typed_chars() {
        let (mut termi, _temp_dir, _env_guard) = init("hello world");
        simulate_typing(&mut termi, "hello");

        let positions = calculate_word_positions(&termi.words, 50);
        let typing_text = TypingAreaComponent::create_text(&termi, 0, 3, &positions);
        let chars_and_styles = get_all_chars_and_styles(&typing_text);

        let theme = termi.current_theme();
        let expected_success_style = Style::default().fg(theme.success());

        #[allow(clippy::needless_range_loop)]
        for i in 0..5 {
            let (ch, style) = chars_and_styles[i];
            assert_eq!(ch, "hello".chars().nth(i).unwrap());
            assert_eq!(style.fg, expected_success_style.fg,);
        }

        let expected_dim_style = Style::default().fg(theme.fg()).add_modifier(Modifier::DIM);

        #[allow(clippy::needless_range_loop)]
        for i in 6..chars_and_styles.len() {
            let (_, style) = chars_and_styles[i];
            assert_eq!(style.fg, expected_dim_style.fg,);
        }
    }

    #[test]
    fn test_ui_handles_wrongly_typed_chars() {
        let (mut termi, _temp_dir, _env_guard) = init("hello world");
        simulate_typing(&mut termi, "hallo"); // Wrong: 'a' instead of 'e'

        let positions = calculate_word_positions(&termi.words, 50);
        let typing_text = TypingAreaComponent::create_text(&termi, 0, 3, &positions);
        let chars_and_styles = get_all_chars_and_styles(&typing_text);

        let theme = termi.current_theme();
        let expected_success_style = Style::default().fg(theme.success());
        let expected_error_style = Style::default().fg(theme.error());

        // 'h' should be correct
        let (h_char, h_style) = chars_and_styles[0];
        assert_eq!(h_char, 'h');
        assert_eq!(h_style.fg, expected_success_style.fg);

        // 'a' should be wrong
        let (a_char, a_style) = chars_and_styles[1];
        assert_eq!(a_char, 'e');
        assert_eq!(a_style.fg, expected_error_style.fg,);
    }

    #[test]
    fn test_ui_char_desync_with_accented_chars() {
        let (mut termi, _temp_dir, _env_guard) = init("sí prueba");

        termi.tracker.start_typing();

        // Type 's'
        termi.tracker.type_char('s');
        let positions = calculate_word_positions(&termi.words, 50);
        let typing_text = TypingAreaComponent::create_text(&termi, 0, 3, &positions);
        let chars_and_styles = get_all_chars_and_styles(&typing_text);

        let theme = termi.current_theme();
        let expected_success_style = Style::default().fg(theme.success());

        // 's'
        let (s_char, s_style) = chars_and_styles[0];
        assert_eq!(s_char, 's');
        assert_eq!(s_style.fg, expected_success_style.fg,);

        // 'í'
        termi.tracker.type_char('í');
        let typing_text = TypingAreaComponent::create_text(&termi, 0, 3, &positions);
        let chars_and_styles = get_all_chars_and_styles(&typing_text);

        let (i_char, i_style) = chars_and_styles[1];
        assert_eq!(i_char, 'í');
        assert_eq!(i_style.fg, expected_success_style.fg,);

        // <space>
        termi.tracker.type_char(' ');

        // 'p'
        termi.tracker.type_char('p');
        let typing_text = TypingAreaComponent::create_text(&termi, 0, 3, &positions);
        let chars_and_styles = get_all_chars_and_styles(&typing_text);

        let (p_char, p_style) = chars_and_styles[3];
        assert_eq!(p_char, 'p');
        assert_eq!(p_style.fg, expected_success_style.fg);
    }

    #[test]
    fn test_ui_and_tracker_sync() {
        let (mut termi, _temp_dir, _env_guard) = init("test word");
        simulate_typing(&mut termi, "test ");

        assert_eq!(termi.tracker.cursor_position, 5);
        assert_eq!(termi.tracker.user_input.len(), 5);

        // check tracker character positining
        for i in 0..5 {
            let tracker_char = termi.tracker.user_input[i];
            let target_char = termi.tracker.target_chars[i];
            assert_eq!(tracker_char, Some(target_char));
        }

        // check ui rendering matches tracker positions
        let positions = calculate_word_positions(&termi.words, 50);
        let typing_text = TypingAreaComponent::create_text(&termi, 0, 3, &positions);
        let chars_and_styles = get_all_chars_and_styles(&typing_text);

        let theme = termi.current_theme();
        let expected_success_style = Style::default().fg(theme.success());

        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            let (ui_char, ui_style) = chars_and_styles[i];
            let target_char = termi.tracker.target_chars[i];

            assert_eq!(ui_char, target_char,);
            assert_eq!(ui_style.fg, expected_success_style.fg,);
        }
    }

    #[test]
    fn test_ui_with_backspace_corrections() {
        let (mut termi, _temp_dir, _env_guard) = init("hello");

        termi.tracker.start_typing();
        termi.tracker.type_char('h');
        termi.tracker.type_char('a'); // wrong
        termi.tracker.backspace(); // fix
        termi.tracker.type_char('e'); // correct

        let positions = calculate_word_positions(&termi.words, 50);
        let typing_text = TypingAreaComponent::create_text(&termi, 0, 3, &positions);
        let chars_and_styles = get_all_chars_and_styles(&typing_text);

        let theme = termi.current_theme();
        let expected_success_style = Style::default().fg(theme.success());

        let (h_char, h_style) = chars_and_styles[0];
        assert_eq!(h_char, 'h');
        assert_eq!(h_style.fg, expected_success_style.fg);

        let (e_char, e_style) = chars_and_styles[1];
        assert_eq!(e_char, 'e');
        assert_eq!(e_style.fg, expected_success_style.fg,);
    }
}
