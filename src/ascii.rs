use crate::assets::ASSETS;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Ascii {
    pub name: String,
}

pub fn get_ascii(name: &str) -> Option<String> {
    ASSETS
        .get_file(format!("ascii/{name}.txt"))
        .map(|f| f.contents_utf8().unwrap_or_default().to_string())
}

pub fn list_ascii() -> Vec<String> {
    ASSETS
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
        .unwrap_or_default()
}

pub fn get_default_art_by_os() -> &'static str {
    match () {
        _ if cfg!(target_os = "macos") => "Apple",
        _ if cfg!(target_os = "windows") => "Windows7",
        _ if cfg!(target_os = "linux") => "Linux",
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
