use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::ListItem,
};

use crate::{
    menu::{MenuItem, MenuItemResult},
    termi::Termi,
    theme::Theme,
};

pub struct MenuHelpers;

impl MenuHelpers {
    pub fn build_menu_items<'a>(
        termi: &'a Termi,
        scroll_offset: usize,
        max_visible: usize,
        hide_description: bool,
    ) -> (Vec<ListItem<'a>>, usize) {
        let theme = termi.current_theme().clone();

        let Some(menu) = termi.menu.current_menu() else {
            return (vec![ListItem::new("  No menu content")], 0);
        };

        let items = menu.items();
        let total_items = items.len();

        if total_items == 0 {
            let no_matches = vec![
                ListItem::new(""),
                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(
                        "grep: pattern not found",
                        Style::default()
                            .fg(theme.muted())
                            .add_modifier(Modifier::DIM),
                    ),
                ])),
            ];
            return (no_matches, 0);
        }

        let current_item_id = menu
            .current_item()
            .map(|i| i.id.clone())
            .unwrap_or_default();

        let visible_items: Vec<_> = items
            .iter()
            .skip(scroll_offset)
            .take(max_visible)
            .cloned()
            .collect();

        let max_key_width = visible_items
            .iter()
            .filter_map(|item| item.key.as_ref())
            .map(|key_text| key_text.chars().count())
            .max()
            .unwrap_or(0);

        let list_items: Vec<ListItem<'a>> = std::iter::once(ListItem::new(""))
            .chain(visible_items.iter().map(|item| {
                let is_selected = item.id == current_item_id;
                let spans = Self::create_item_spans(
                    item,
                    is_selected,
                    max_key_width,
                    hide_description,
                    termi.config.hide_cursorline,
                    &theme,
                );
                ListItem::new(Line::from(spans))
            }))
            .collect();

        (list_items, total_items)
    }

    fn create_item_spans(
        item: &MenuItem,
        is_selected: bool,
        max_key_width: usize,
        hide_description: bool,
        hide_cursorline: bool,
        theme: &Theme,
    ) -> Vec<Span<'static>> {
        let arrow_symbol = "❯ ";
        let submenu_symbol = " →";

        let should_render_cursorline = is_selected && !item.is_disabled && !hide_cursorline;
        let content_bg =
            Self::get_content_bg(is_selected, item.is_disabled, hide_cursorline, theme);

        let mut spans = vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                if is_selected { arrow_symbol } else { "  " },
                Style::default()
                    .fg(if is_selected {
                        theme.success()
                    } else {
                        theme.fg()
                    })
                    .bg(content_bg),
            ),
        ];

        if let Some(key_text) = &item.key {
            // Info items (kv pairs)
            let formatted_key = if hide_description {
                key_text.to_string()
            } else {
                format!("{:<width$}", key_text, width = max_key_width + 2)
            };
            spans.push(Span::styled(
                formatted_key,
                Style::default()
                    .fg(theme.accent())
                    .bg(content_bg)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                item.label.clone(),
                Style::default().fg(theme.fg()).bg(content_bg),
            ));
        } else {
            let label_style =
                Self::get_label_style(item, is_selected, should_render_cursorline, theme);

            // toggleable checkbox
            if let Some(is_active) = item.is_active {
                let checkbox_text = if is_active { "[✓] " } else { "[ ] " };
                let checkbox_style = if is_active {
                    Style::default().fg(theme.success()).bg(content_bg)
                } else {
                    Style::default()
                        .fg(theme.border())
                        .bg(content_bg)
                        .add_modifier(Modifier::DIM)
                };
                spans.push(Span::styled(checkbox_text, checkbox_style));
            }

            spans.push(Span::styled(item.label.clone(), label_style.bg(content_bg)));
        }

        // sub-menu arrow
        if matches!(item.result, MenuItemResult::OpenSubMenu(_)) {
            spans.push(Span::styled(
                submenu_symbol,
                Style::default().fg(theme.primary()).bg(content_bg),
            ));
        }

        spans
    }

    fn get_content_bg(
        is_selected: bool,
        is_disabled: bool,
        hide_cursorline: bool,
        theme: &Theme,
    ) -> Color {
        if is_selected && !is_disabled && !hide_cursorline {
            theme.selection_bg()
        } else {
            theme.bg()
        }
    }

    fn get_label_style(
        item: &MenuItem,
        is_selected: bool,
        should_render_cursorline: bool,
        theme: &Theme,
    ) -> Style {
        if is_selected && !should_render_cursorline {
            Style::default()
                .fg(theme.success())
                .add_modifier(Modifier::BOLD)
        } else if is_selected && should_render_cursorline {
            Style::default()
                .fg(theme.selection_fg())
                .bg(theme.selection_bg())
        } else if item.is_disabled {
            Style::default()
                .fg(theme.muted())
                .add_modifier(Modifier::DIM)
        } else {
            match &item.result {
                MenuItemResult::OpenSubMenu(_) => Style::default().fg(theme.fg()),
                MenuItemResult::ToggleState => {
                    if item.is_active == Some(true) {
                        Style::default().fg(theme.success())
                    } else {
                        Style::default()
                            .fg(theme.muted())
                            .add_modifier(Modifier::DIM)
                    }
                }
                _ => Style::default().fg(theme.fg()),
            }
        }
    }
}
