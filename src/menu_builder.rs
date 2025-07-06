use crate::{
    actions::{MenuContext, PreviewType, TermiAction},
    ascii,
    config::{Config, ModeType},
    constants::{DEFAULT_TIME_DURATION_LIST, DEFAULT_WORD_COUNT_LIST},
    menu::{Menu, MenuItem},
    modal::ModalContext,
    styles::{PickerStyle, ResultsStyle, TermiStyle},
    version::VERSION,
};

pub fn build_menu(ctx: MenuContext, config: &Config) -> Menu {
    match ctx {
        MenuContext::Root => build_root_menu(config),
        MenuContext::Theme => build_theme_menu(),
        MenuContext::Language => build_language_menu(),
        MenuContext::Cursor => build_cursor_menu(),
        MenuContext::PickerStyle => build_picker_style_menu(),
        MenuContext::Mode => build_mode_menu(),
        MenuContext::Time => build_time_menu(),
        MenuContext::Words => build_words_menu(),
        MenuContext::LineCount => build_lines_count_menu(),
        MenuContext::Help => build_help_menu(),
        MenuContext::About => build_about_menu(),
        MenuContext::AsciiArt => build_ascii_art_menu(),
        MenuContext::Options => build_options_menu(config),
        MenuContext::Results => build_results_style_menu(),
        MenuContext::Leaderboard => build_leaderboard_menu(),
    }
}

fn build_root_menu(config: &Config) -> Menu {
    let items = vec![
        MenuItem::sub_menu("root/mode", "Mode...", MenuContext::Mode),
        MenuItem::sub_menu("root/time", "Time...", MenuContext::Time),
        MenuItem::sub_menu("root/words", "Words...", MenuContext::Words),
        MenuItem::sub_menu("root/language", "Language...", MenuContext::Language),
        MenuItem::sub_menu("root/options", "Options...", MenuContext::Options),
        MenuItem::sub_menu("root/theme", "Theme...", MenuContext::Theme)
            .disabled(!config.term_has_color_support()),
        MenuItem::sub_menu("root/picker", "Picker Style...", MenuContext::PickerStyle),
        MenuItem::sub_menu("root/results", "Result Style...", MenuContext::Results),
        MenuItem::action(
            "root/leaderboard",
            "Leaderboard...",
            TermiAction::LeaderboardOpen,
        )
        .disabled(config.no_track),
        MenuItem::sub_menu("root/ascii", "Neofetch Art...", MenuContext::AsciiArt),
        MenuItem::sub_menu("root/lines", "Visible Lines...", MenuContext::LineCount),
        MenuItem::sub_menu("root/cursor", "Cursor Style...", MenuContext::Cursor),
        MenuItem::sub_menu("root/help", "Help...", MenuContext::Help),
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
                &format!("themes/{name}"),
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
                &format!("lang/{lang}"),
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
                &format!("cursor/{style}"),
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

// Change picker style menu
fn build_picker_style_menu() -> Menu {
    let styles = PickerStyle::all();
    let items = styles
        .iter()
        .map(|&style| {
            MenuItem::action(
                &format!("picker/{style}"),
                PickerStyle::label_from_str(style),
                TermiAction::ChangePickerStyle(style.to_string()),
            )
        })
        .collect();
    Menu::new(
        MenuContext::PickerStyle,
        "Select Picker Style".to_string(),
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
    let times = DEFAULT_TIME_DURATION_LIST;
    let mut items: Vec<MenuItem> = times
        .iter()
        .map(|&t| {
            MenuItem::action(
                &format!("time/{t}"),
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
    let counts = DEFAULT_WORD_COUNT_LIST;
    let mut items: Vec<MenuItem> = counts
        .iter()
        .map(|&c| {
            MenuItem::action(
                &format!("words/{c}"),
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

/// Builds the Ascii Art menu
fn build_ascii_art_menu() -> Menu {
    let arts = ascii::available_ascii_arts();
    let items: Vec<MenuItem> = arts
        .iter()
        .map(|name| {
            MenuItem::action(
                &format!("ascii/{name}"),
                name,
                TermiAction::ChangeAsciiArt(name.to_string()),
            )
            .with_preview(PreviewType::AsciiArt(name.to_string()))
        })
        .collect();
    Menu::new(MenuContext::AsciiArt, "Select ASCII".to_string(), items)
}

// Visible Line count menu
fn build_lines_count_menu() -> Menu {
    let lines = [1, 2, 3, 4, 5];
    let items: Vec<MenuItem> = lines
        .iter()
        .map(|&line_count| {
            MenuItem::action(
                &format!("lines/{line_count}"),
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

/// Builds the Options menu with all toggleable settings
fn build_options_menu(config: &Config) -> Menu {
    let items = vec![
        // === Test Content ===
        MenuItem::toggle("options/symbols", "Use Symbols", config.use_symbols),
        MenuItem::toggle(
            "options/punctuation",
            "Use Punctuation",
            config.use_punctuation,
        ),
        MenuItem::toggle("options/numbers", "Use Numbers", config.use_numbers),
        // === Display Options ===
        MenuItem::toggle(
            "options/show_live_wpm",
            "Show Live WPM",
            !config.hide_live_wpm,
        ),
        MenuItem::toggle("options/fps", "Show FPS", config.show_fps),
        MenuItem::toggle(
            "options/show_cursorline",
            "Show Menu Cursorline",
            !config.hide_cursorline,
        ),
        MenuItem::toggle(
            "options/show_notifications",
            "Show Notifications",
            !config.hide_notifications,
        ),
        MenuItem::toggle(
            "options/monochromatic",
            "Monochromatic Results",
            config.monocrhomatic_results,
        ),
    ];
    Menu::new(MenuContext::Options, "Options".to_string(), items)
}

/// Builds the Help menu
fn build_help_menu() -> Menu {
    let lines = [
        // === General ===
        "[all] F1 -> Toggle Help",
        "[all] F2 -> Toggle Themes",
        "[all] esc -> Toggle Menu",
        "[all] ctrl-c -> Quit",
        "[all] ctrl-z -> Quit (alt)",
        "[all] tab-enter -> Restart Test",
        // === Menu Nav ===
        "[menu] ↑/k -> Navigate Up",
        "[menu] ↓/j -> Navigate Down",
        "[menu] gg -> Go to Top",
        "[menu] shift-g -> Go to Bottom",
        "[menu] ctrl-u -> Half Page Up",
        "[menu] ctrl-d -> Half Page Down",
        "[menu] l/enter -> Select Item / Open Submenu",
        "[menu] h/esc -> Go Back / Close Menu",
        "[menu] ctrl-y -> Select Item / Open Submenu",
        "[menu] space -> Toggle Option",
        "[menu] / -> Start Search Mode",
        // === Menu Search ===
        "[search] enter -> Confirm Search/Select",
        "[search] esc -> Exit Search mode",
        "[search] ctrl-p -> Navigate Up",
        "[search] ctrl-n -> Navigate Down",
        "[search] ctrl-k -> Navigate Up",
        "[search] ctrl-j -> Navigate Down",
        // === Results ===
        "[results] r -> Redo Test",
        "[results] n -> Start New Test",
        "[results] q -> Quit Application",
        "[results] esc -> Toggle Menu",
    ];

    let parsed_entries: Vec<(String, String, String)> = lines
        .iter()
        .map(|&line| {
            let parts: Vec<&str> = line.splitn(2, " -> ").collect();
            let full_key = parts[0].trim();
            let description = parts.get(1).map_or("", |d| d.trim());

            // split the parts (context, keybind, description)
            if let Some(bracket_end) = full_key.rfind(']') {
                let context = &full_key[..=bracket_end];
                let keybind = full_key[bracket_end + 1..].trim();
                (
                    context.to_string(),
                    keybind.to_string(),
                    description.to_string(),
                )
            } else {
                (String::new(), full_key.to_string(), description.to_string())
            }
        })
        .collect();

    let max_total_width = parsed_entries
        .iter()
        .map(|(context, keybind, _)| context.chars().count() + keybind.chars().count())
        .max()
        .unwrap_or(0);

    let items: Vec<MenuItem> = parsed_entries
        .iter()
        .enumerate()
        .map(|(idx, (context, keybind, description))| {
            let total_key_part = format!("{context}{keybind}");
            let formatted_key = format!("{total_key_part:<max_total_width$}");

            let item_id = format!("help/{idx}");
            MenuItem::info(&item_id, &formatted_key, description)
        })
        .collect();

    Menu::new(MenuContext::Help, "Keybinds".to_string(), items)
}

/// Builds the About menu
fn build_about_menu() -> Menu {
    let items = vec![
        MenuItem::info("about/name", "Name", "termitype"),
        MenuItem::info("about/version", "Version", VERSION),
        MenuItem::info("about/license", "License", env!("CARGO_PKG_LICENSE")),
        MenuItem::info("about/desc", "Description", "TUI typing game"),
        MenuItem::info(
            "about/source",
            "Source",
            "https://github.com/emanuel2718/termitype",
        ),
    ];
    Menu::new(MenuContext::About, "About Termitype".to_string(), items)
}

/// Builds the Results menu
fn build_results_style_menu() -> Menu {
    let styles = ResultsStyle::all();
    let items = styles
        .iter()
        .map(|&style| {
            MenuItem::action(
                &format!("results/{style}"),
                ResultsStyle::label_from_str(style),
                TermiAction::ChangeResultsStyle(style.to_string()),
            )
            .with_preview(PreviewType::ResultsStyle(
                style
                    .parse::<ResultsStyle>()
                    .unwrap_or(ResultsStyle::default()),
            ))
        })
        .collect();
    Menu::new(MenuContext::Results, "Results Style".to_string(), items)
}

fn build_leaderboard_menu() -> Menu {
    let items = vec![MenuItem::action(
        "leaderboard/open",
        "Open Leaderboard",
        TermiAction::LeaderboardOpen,
    )];

    Menu::new(MenuContext::Leaderboard, "Leaderboard".to_string(), items)
}
