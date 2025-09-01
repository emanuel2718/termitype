use std::time;

const SECS_PER_MIN: u64 = 60;
const SECS_PER_HOUR: u64 = 3_600;
const SECS_PER_DAY: u64 = 86_400;
const DAYS_PER_YEAR: u64 = 365;
const DAYS_PER_MONTH: u64 = 30;

/// Formats the give time in y-m-dThh-mm-ss.mmm
pub fn format_timestamp(time: time::SystemTime) -> String {
    let duration = time.duration_since(time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let ms = duration.as_millis();

    let days_since_epoch = secs / SECS_PER_DAY;
    let secs_in_day = secs % SECS_PER_DAY;

    let years_since_epoch = days_since_epoch / DAYS_PER_YEAR;
    let remaining_days = days_since_epoch % DAYS_PER_YEAR;

    let month = (remaining_days / DAYS_PER_MONTH) + 1;
    let day = (remaining_days % DAYS_PER_MONTH) + 1;
    let year = 1970 + years_since_epoch;

    let hour = secs_in_day / SECS_PER_HOUR;
    let min = (secs_in_day % SECS_PER_HOUR) / SECS_PER_MIN;
    let sec = secs_in_day % SECS_PER_MIN;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}.{ms:03}")
}

// TODO: maybe we can improve this to be more performant. Using the most basic fuzzy search possible for now
pub fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let text = text.chars().collect::<Vec<_>>();
    let pattern = pattern.chars().collect::<Vec<_>>();

    let mut text_idx = 0;
    let mut pattern_idx = 0;

    while text_idx < text.len() && pattern_idx < pattern.len() {
        if text[text_idx] == pattern[pattern_idx] {
            pattern_idx += 1;
        }
        text_idx += 1;
    }

    pattern_idx == pattern.len()
}
