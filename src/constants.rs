pub const APPNAME: &str = env!("CARGO_PKG_NAME");
pub const DEFAULT_LANGUAGE: &str = "english";
pub const DEFAULT_CURSOR_STYLE: &str = "blinking-beam";
pub const DEFAULT_THEME: &str = "tokyonight";
// pub const DEFAULT_THEME: &str = "termitype-dark";

pub const STATE_FILE: &str = ".state";
pub const LOG_FILE: &str = "debug.log";

pub const DEBUG_KEY: char = 'd';
pub const BACKSPACE: char = '\x08';

pub const AMOUNT_OF_VISIBLE_LINES: u8 = 3;

// ui
pub const WINDOW_WIDTH_PERCENT: u16 = 80;
pub const WINDOW_HEIGHT_PERCENT: u16 = 90;

pub const MIN_WIDTH: u16 = 20;
pub const MIN_HEIGHT: u16 = 10;

pub const MENU_WIDTH: u16 = 45;
pub const MENU_HEIGHT: u16 = 20;

pub const TOP_BAR_HEIGHT: u16 = 8;
pub const MODE_BAR_OFFSET: u16 = 1;
pub const COMMAND_BAR_HEIGHT: u16 = 4;
pub const FOOTER_HEIGHT: u16 = 1;

pub const MIN_TYPING_HEIGHT: u16 = 3;
pub const TYPING_AREA_WIDTH_PERCENT: u16 = 80;

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

// TODO: dont be lazy and improve the naming of this constants
pub const SMALL_SCREEN_WIDTH: u16 = 85;
pub const SMALL_SCREEN_HEIGHT: u16 = 20;
pub const MIN_WIDTH_TO_SHOW_FOOTER: u16 = 55;
pub const MIN_TERM_HEIGHT: u16 = 15;
pub const MIN_TERM_WIDTH: u16 = 15;
