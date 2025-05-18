pub const APPNAME: &str = env!("CARGO_PKG_NAME");
pub const DEFAULT_LINE_COUNT: u8 = 3;
pub const DEFAULT_LANGUAGE: &str = "english";
pub const DEFAULT_CURSOR_STYLE: &str = "blinking-beam";
pub const DEFAULT_THEME: &str = "tokyonight";
// pub const DEFAULT_THEME: &str = "termitype-dark";

pub const DEFAULT_TIME_MODE_DURATION: usize = 30;
pub const DEFAULT_TIME_DURATION_LIST: [usize; 4] = [15, 30, 60, 120];

// This is the target wps that we use. This assument as constant 350wpm as our upper bound
pub const WPS_TARGET: f64 = 6.0;

pub const DEFAULT_WORD_MODE_COUNT: usize = 50;
pub const DEFAULT_WORD_COUNT_LIST: [usize; 4] = [10, 25, 50, 100];

pub const STATE_FILE: &str = ".state";
pub const LOG_FILE: &str = "debug.log";

pub const BACKSPACE_CHAR: char = '\x08';

// TODO: move to a seperate file with other arts that the user can choose from or will use a sane default
pub const ASCII_ART: &str = r"
    ⠄⠄⠄⠄⠄⠄⠄⢀⣠⣶⣾⣿⣶⣦⣤⣀⠄⢀⣀⣤⣤⣤⣤⣄⠄⠄⠄⠄⠄⠄
    ⠄⠄⠄⠄⠄⢀⣴⣿⣿⣿⡿⠿⠿⠿⠿⢿⣷⡹⣿⣿⣿⣿⣿⣿⣷⠄⠄⠄⠄⠄
    ⠄⠄⠄⠄⠄⣾⣿⣿⣿⣯⣵⣾⣿⣿⡶⠦⠭⢁⠩⢭⣭⣵⣶⣶⡬⣄⣀⡀⠄⠄
    ⠄⠄⠄⡀⠘⠻⣿⣿⣿⣿⡿⠟⠩⠶⠚⠻⠟⠳⢶⣮⢫⣥⠶⠒⠒⠒⠒⠆⠐⠒
    ⠄⢠⣾⢇⣿⣿⣶⣦⢠⠰⡕⢤⠆⠄⠰⢠⢠⠄⠰⢠⠠⠄⡀⠄⢊⢯⠄⡅⠂⠄
    ⢠⣿⣿⣿⣿⣿⣿⣿⣏⠘⢼⠬⠆⠄⢘⠨⢐⠄⢘⠈⣼⡄⠄⠄⡢⡲⠄⠂⠠⠄
    ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣥⣀⡁⠄⠘⠘⠘⢀⣠⣾⣿⢿⣦⣁⠙⠃⠄⠃⠐⣀
    ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣋⣵⣾⣿⣿⣿⣿⣦⣀⣶⣾⣿⣿⡉⠉⠉
    ⣿⣿⣿⣿⣿⣿⣿⠟⣫⣥⣬⣭⣛⠿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡆⠄
    ⣿⣿⣿⣿⣿⣿⣿⠸⣿⣏⣙⠿⣿⣿⣶⣦⣍⣙⠿⠿⠿⠿⠿⠿⠿⠿⣛⣩⣶⠄
    ⣛⣛⣛⠿⠿⣿⣿⣿⣮⣙⠿⢿⣶⣶⣭⣭⣛⣛⣛⣛⠛⠛⠻⣛⣛⣛⣛⣋⠁⢀
    ⣿⣿⣿⣿⣿⣶⣬⢙⡻⠿⠿⣷⣤⣝⣛⣛⣛⣛⣛⣛⣛⣛⠛⠛⣛⣛⠛⣡⣴⣿
    ⣛⣛⠛⠛⠛⣛⡑⡿⢻⢻⠲⢆⢹⣿⣿⣿⣿⣿⣿⠿⠿⠟⡴⢻⢋⠻⣟⠈⠿⠿
    ⣿⡿⡿⣿⢷⢤⠄⡔⡘⣃⢃⢰⡦⡤⡤⢤⢤⢤⠒⠞⠳⢸⠃⡆⢸⠄⠟⠸⠛⢿
    ⡟⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠁⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⢸
";

pub const SECS_PER_MIN: u64 = 60;
pub const SECS_PER_HOUR: u64 = 3_600;
pub const SECS_PER_DAY: u64 = 86_400;
pub const DAYS_PER_YEAR: u64 = 365;
pub const DAYS_PER_MONTH: u64 = 30;

// ui
pub const SMALL_TERM_WIDTH: u16 = 85;
pub const SMALL_TERM_HEIGHT: u16 = 20;

pub const SMALL_RESULTS_WIDTH: u16 = 60;
pub const SMALL_RESULTS_HEIGHT: u16 = 20;

pub const MIN_TERM_HEIGHT: u16 = 15;
pub const MIN_TERM_WIDTH: u16 = 25;

pub const MIN_FOOTER_WIDTH: u16 = 55;
pub const MIN_THEME_PREVIEW_WIDTH: u16 = 60;

pub const MENU_HEIGHT: u16 = 25;

pub const MODAL_WIDTH: u16 = 50;
pub const MODAL_HEIGHT: u16 = 11;

// top area
pub const HEADER_HEIGHT: u16 = 4;
pub const ACTION_BAR_HEIGHT: u16 = 1;
pub const TOP_AREA_HEIGHT: u16 = HEADER_HEIGHT + ACTION_BAR_HEIGHT;

// mid area
pub const MODE_BAR_HEIGHT: u16 = 2;
pub const TYPING_AREA_WIDTH: u16 = 80;

// bottom area
pub const COMMAND_BAR_HEIGHT: u16 = 3;
pub const FOOTER_HEIGHT: u16 = 1;
pub const BOTTOM_PADDING: u16 = 1;
pub const BOTTOM_AREA_HEIGHT: u16 = COMMAND_BAR_HEIGHT + BOTTOM_PADDING + FOOTER_HEIGHT;

// Modals
pub const MIN_CUSTOM_TIME: u16 = 1;
pub const MAX_CUSTOM_TIME: u16 = 300; // 5 minutes

pub const MIN_CUSTOM_WORD_COUNT: u16 = 1;
pub const MAX_CUSTOM_WORD_COUNT: u16 = 5000;
