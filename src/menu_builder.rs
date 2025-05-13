use crate::{
    actions::{MenuContext, PreviewType, TermiAction},
    config::{Config, ModeType},
    menu::{Menu, MenuItem},
    modal::ModalContext,
    version::VERSION,
};

pub fn build_menu(ctx: MenuContext, config: &Config) -> Menu {
    match ctx {
        MenuContext::Root => build_root_menu(config),
        MenuContext::Theme => build_theme_menu(),
        MenuContext::Language => build_language_menu(),
        MenuContext::Cursor => build_cursor_menu(),
        MenuContext::Mode => build_mode_menu(),
        MenuContext::Time => build_time_menu(),
        MenuContext::Words => build_words_menu(),
        MenuContext::LineCount => build_lines_count_menu(),
        MenuContext::About => build_about_menu(),
    }
}

fn build_root_menu(config: &Config) -> Menu {
    let items = vec![
        MenuItem::toggle("root/punctuation", "Punctuation", config.use_punctuation),
        MenuItem::toggle("root/numbers", "Numbers", config.use_numbers),
        MenuItem::toggle("root/symbols", "Symbols", config.use_symbols),
        MenuItem::sub_menu("root/mode", "Mode...", MenuContext::Mode),
        MenuItem::sub_menu("root/time", "Time...", MenuContext::Time),
        MenuItem::sub_menu("root/words", "Words...", MenuContext::Words),
        MenuItem::sub_menu("root/language", "Language...", MenuContext::Language),
        MenuItem::sub_menu("root/theme", "Theme...", MenuContext::Theme)
            .disabled(!config.term_has_color_support()),
        MenuItem::sub_menu("root/cursor", "Cursor...", MenuContext::Cursor),
        MenuItem::sub_menu("root/lines", "Visible Lines...", MenuContext::LineCount),
        MenuItem::sub_menu("root/about", "About...", MenuContext::About),
        MenuItem::action("root/quit", "Exit", TermiAction::Quit),
    ];
    Menu::new(MenuContext::Root, "Main Menu".to_string(), items)
}

// Chnage theme menu
fn build_theme_menu() -> Menu {
    let themes = crate::theme::available_themes();
    let items: Vec<MenuItem> = themes
        .iter()
        .map(|name| {
            MenuItem::action(
                &format!("themes/{}", name),
                name,
                TermiAction::ChangeTheme(name.to_string()),
            )
            .with_preview(PreviewType::Theme(name.to_string()))
        })
        .collect();
    Menu::new(MenuContext::Theme, "Select Theme".to_string(), items)
}

// Chnage test language menu
fn build_language_menu() -> Menu {
    let languages = crate::builder::Builder::available_languages();
    let items = languages
        .iter()
        .map(|lang| {
            MenuItem::action(
                &format!("lang/{}", lang),
                lang,
                TermiAction::ChangeLanguage(lang.to_string()),
            )
        })
        .collect();
    Menu::new(MenuContext::Language, "Select Language".to_string(), items)
}

// Chnage cursor style menu
fn build_cursor_menu() -> Menu {
    let styles = [
        "beam",
        "block",
        "underline",
        "blinking-beam",
        "blinking-block",
        "blinking-underline",
    ];
    let items = styles
        .iter()
        .map(|&style| {
            MenuItem::action(
                &format!("cursor/{}", style),
                style,
                TermiAction::ChangeCursor(style.to_string()),
            )
            .with_preview(PreviewType::Cursor(style.to_string()))
        })
        .collect();
    Menu::new(
        MenuContext::Cursor,
        "Select Cursor Style".to_string(),
        items,
    )
}

// Change test mode menu
fn build_mode_menu() -> Menu {
    let items = vec![
        MenuItem::action(
            "mode/time",
            "Time",
            TermiAction::ChangeMode(ModeType::Time, None),
        ),
        MenuItem::action(
            "mode/words",
            "Words",
            TermiAction::ChangeMode(ModeType::Words, None),
        ),
    ];
    Menu::new(MenuContext::Mode, "Select Mode".to_string(), items)
}

// Change test duration menu
fn build_time_menu() -> Menu {
    let times = [15, 30, 60, 120];
    let mut items: Vec<MenuItem> = times
        .iter()
        .map(|&t| {
            MenuItem::action(
                &format!("time/{}", t),
                &t.to_string(),
                TermiAction::ChangeTime(t as u64),
            )
        })
        .collect();
    items.push(MenuItem::action(
        "time/custom",
        "Custom...",
        TermiAction::ModalOpen(ModalContext::CustomTime),
    ));
    Menu::new(MenuContext::Time, "Select Time".to_string(), items)
}

// Change word count menu
fn build_words_menu() -> Menu {
    let counts = [10, 25, 50, 100];
    let mut items: Vec<MenuItem> = counts
        .iter()
        .map(|&c| {
            MenuItem::action(
                &format!("words/{}", c),
                &c.to_string(),
                TermiAction::ChangeWordCount(c),
            )
        })
        .collect();
    items.push(MenuItem::action(
        "words/custom",
        "Custom...",
        TermiAction::ModalOpen(ModalContext::CustomWordCount),
    ));
    Menu::new(MenuContext::Words, "Select Word Count".to_string(), items)
}
// Visible Line count menu
fn build_lines_count_menu() -> Menu {
    let lines = [1, 2, 3, 4, 5];
    let items: Vec<MenuItem> = lines
        .iter()
        .map(|&line_count| {
            MenuItem::action(
                &format!("lines/{}", line_count),
                &line_count.to_string(),
                TermiAction::ChangeVisibleLines(line_count),
            )
        })
        .collect();
    Menu::new(
        MenuContext::LineCount,
        "Select Visible Lines".to_string(),
        items,
    )
}

// About menu
fn build_about_menu() -> Menu {
    let items = vec![
        MenuItem::action("about/name", "Name: termitype", TermiAction::NoOp),
        MenuItem::action(
            "about/version",
            &format!("Version: {}", VERSION),
            TermiAction::NoOp,
        ),
        MenuItem::action(
            "about/desc",
            "Description: A typing game for the terminal.",
            TermiAction::NoOp,
        ),
        MenuItem::action(
            "about/source",
            "Source: http://github.com/emanuel2718/termitype",
            TermiAction::NoOp,
        ),
    ];
    Menu::new(MenuContext::About, "About Termitype".to_string(), items)
}
