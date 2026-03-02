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

/// Truncate a string to a maximum display width
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut current_width = 0;

    for c in s.chars() {
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if current_width + char_width > max_width {
            break;
        }
        result.push(c);
        current_width += char_width;
    }

    result
}

/// Case-insensitive subsequence fuzzy match.
/// Returns true if every character in `pattern` appears in `text` in order.
pub fn fuzzy_match(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    let mut pattern_chars = pattern.chars().flat_map(char::to_lowercase);
    let mut current = pattern_chars.next();

    for ch in text.chars().flat_map(char::to_lowercase) {
        if let Some(c) = current
            && ch == c
        {
            current = pattern_chars.next();
            if current.is_none() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::fuzzy_match;

    #[test]
    fn empty_pattern_always_matches() {
        assert!(fuzzy_match("anything", ""));
        assert!(fuzzy_match("", ""));
    }

    #[test]
    fn empty_text_no_match() {
        assert!(!fuzzy_match("", "a"));
    }

    #[test]
    fn exact_match() {
        assert!(fuzzy_match("termitype dark", "termitype dark"));
        assert!(fuzzy_match("termitype light", "termitype light"));
    }

    #[test]
    fn subsequence_match() {
        assert!(fuzzy_match("termitype dark", "ttd"));
        assert!(fuzzy_match("termitype dark", "tdk"));
        assert!(fuzzy_match("termitype light", "ttl"));
        assert!(fuzzy_match("one-dark-pro", "odp"));
    }

    #[test]
    fn case_insensitive() {
        assert!(fuzzy_match("Termitype Dark", "ttd"));
        assert!(fuzzy_match("termitype dark", "TTD"));
        assert!(fuzzy_match("TERMITYPE LIGHT", "ttl"));
        assert!(fuzzy_match("Termitype Light", "TTL"));
    }

    #[test]
    fn no_match() {
        assert!(!fuzzy_match("termitype dark", "xyz"));
        assert!(!fuzzy_match("abc", "abcd"));
        assert!(!fuzzy_match("ab", "ba"));
    }

    #[test]
    fn theme_name_examples() {
        assert!(fuzzy_match("nord", "no"));
        assert!(fuzzy_match("gruvbox-dark", "gbd"));
        assert!(fuzzy_match("Solarized Light", "sl"));
        assert!(!fuzzy_match("nord", "nz"));
    }
}
