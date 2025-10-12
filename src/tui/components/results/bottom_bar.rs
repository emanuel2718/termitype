use crate::{theme::Theme, variants::ResultsVariant};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

const VARIANTS: &[(&str, &str, ResultsVariant)] = &[
    ("Minimal", "m", ResultsVariant::Minimal),
    ("Graph", "g", ResultsVariant::Graph),
    ("Neofetch", "n", ResultsVariant::Neofetch),
];

const ACTIONS: &[(&str, &str)] = &[
    ("New", "N"),
    ("Redo", "r"),
    ("Menu", "esc"),
    ("Quit", "^c"),
    ("Randomize Theme", "^t"),
];

enum FooterEntry {
    Splitter,
    Item(FooterItem),
}

struct FooterItem {
    key: &'static str,
    label: &'static str,
    is_selected: bool,
}

struct FooterState {
    entries: Vec<FooterEntry>,
}

pub fn render_bar(frame: &mut Frame, theme: &Theme, variant: ResultsVariant, area: Rect) {
    let state = calculate_state(variant, area.width);
    let line = build_footer_line(&state, theme);
    let footer_paragraph = Paragraph::new(line)
        .alignment(Alignment::Left)
        .block(ratatui::widgets::Block::default());

    frame.render_widget(footer_paragraph, area);
}

fn calculate_state(current_variant: ResultsVariant, width: u16) -> FooterState {
    let mut entries: Vec<FooterEntry> = Vec::new();
    let mut line_width = 0_usize;

    for (label, key, variant) in VARIANTS.iter() {
        let item = FooterItem {
            label,
            key,
            is_selected: *variant == current_variant,
        };

        let item_width = calculate_item_width(&item);

        if line_width > 0 {
            entries.push(FooterEntry::Splitter);
            line_width += 1;
        }
        line_width += item_width;
        entries.push(FooterEntry::Item(item));
    }

    for (label, key) in ACTIONS.iter() {
        let item = FooterItem {
            label,
            key,
            is_selected: false,
        };
        let item_width = calculate_item_width(&item);

        if line_width + 1 + item_width > width as usize {
            break; // we ran out of space
        }

        entries.push(FooterEntry::Splitter);
        line_width += 1;
        line_width += item_width;
        entries.push(FooterEntry::Item(item));
    }

    FooterState { entries }
}

fn build_footer_line<'a>(state: &'a FooterState, theme: &'a Theme) -> Line<'a> {
    let spans: Vec<Span> = state
        .entries
        .iter()
        .flat_map(|entry| match entry {
            FooterEntry::Item(item) => {
                create_footer_item(item.label, item.key, theme, item.is_selected)
            }
            FooterEntry::Splitter => vec![Span::raw(" ")],
        })
        .collect();

    Line::from(spans)
}

fn create_footer_item<'a>(
    label: &'a str,
    key: &'a str,
    theme: &Theme,
    selected: bool,
) -> Vec<Span<'a>> {
    let (fg_color, bg_color, use_bold) = if selected {
        (theme.cursor_text(), theme.warning(), true)
    } else {
        (theme.selection_fg(), theme.selection_bg(), false)
    };

    let mut style = Style::default().fg(fg_color).bg(bg_color);
    if use_bold {
        style = style.add_modifier(Modifier::BOLD);
    }

    // Format: " Label [k] "
    let content = format!(" {} [{}] ", label, key);

    vec![Span::styled(content, style)]
}

fn calculate_item_width(item: &FooterItem) -> usize {
    // Format: " Label [k] " (with padding inside the box)
    let content = format!(" {} [{}] ", item.label, item.key);
    UnicodeWidthStr::width(content.as_str())
}
