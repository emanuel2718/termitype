#[cfg(not(debug_assertions))]
use crate::assets::ASSETS;
#[cfg(debug_assertions)]
use std::fs;
use std::str::FromStr;

// TODO: add a way to allow custom art loading from `$XDG_CONFIG_HOME/termitype/ascii/{art}.txt`

#[derive(Debug, Clone)]
pub struct Ascii {
    pub name: String,
}

#[cfg(debug_assertions)]
fn load_ascii_from_disk(name: &str) -> Option<String> {
    fs::read_to_string(format!("assets/ascii/{}.txt", name)).ok()
}

#[cfg(debug_assertions)]
fn list_ascii_from_disk() -> Vec<String> {
    fs::read_dir("assets/ascii")
        .ok()
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    entry.path().extension().and_then(|ext| {
                        if ext == "txt" {
                            entry.path().file_stem()?.to_str().map(String::from)
                        } else {
                            None
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn get_ascii(name: &str) -> Option<String> {
    // NOTE(ema): on debug we load the ascii arts directly from disk rather than relying
    // on the `include_dir` macro from `ASSETS`, because adding new arts while developing  required
    // a `cargo clean` to see see the new art, which is annoying
    #[cfg(debug_assertions)]
    {
        load_ascii_from_disk(name)
    }
    #[cfg(not(debug_assertions))]
    {
        ASSETS
            .get_file(format!("ascii/{name}.txt"))
            .map(|f| f.contents_utf8().unwrap_or_default().to_string())
    }
}

pub fn list_ascii() -> Vec<String> {
    #[cfg(debug_assertions)]
    {
        let mut ascii_list: Vec<String> = list_ascii_from_disk();
        ascii_list.sort_by_key(|a| a.to_lowercase());
        ascii_list
    }
    #[cfg(not(debug_assertions))]
    {
        let mut ascii_list: Vec<String> = ASSETS
            .get_dir("ascii")
            .map(|dir| {
                dir.files()
                    .filter(|f| f.path().extension().is_some_and(|ext| ext == "txt"))
                    .filter_map(|f| {
                        f.path()
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .map(String::from)
                    })
                    .collect()
            })
            .unwrap_or_default();
        ascii_list.sort_by_key(|a| a.to_lowercase());
        ascii_list
    }
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
