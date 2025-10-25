pub use crate::assets::{get_ascii, list_ascii};
use std::str::FromStr;

// TODO: add a way to allow custom art loading from `$XDG_CONFIG_HOME/termitype/ascii/{art}.txt`

#[derive(Debug, Clone)]
pub struct Ascii {
    pub name: String,
}

// TODO: we must add support for all the major mainstream linux distributions
// For example, if you are using Void Linux, it would be neat to detect that and show the void art
pub fn get_default_art_by_os() -> &'static str {
    match () {
        _ if cfg!(target_os = "macos") => "Apple",
        _ if cfg!(target_os = "windows") => "Windows7",
        _ if cfg!(target_os = "linux") => "Termitype", // TODO: eventually we want to do per distro art, but default to this for now
        _ => "Termitype",
    }
}

impl FromStr for Ascii {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if get_ascii(s).is_some() {
            Ok(Ascii {
                name: s.to_string(),
            })
        } else {
            Err(format!("Ascii art '{}' not found", s))
        }
    }
}
