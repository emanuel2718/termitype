use crate::constants::DEFAULT_THEME;
use include_dir::{include_dir, Dir};

pub static ASSETS: Dir = include_dir!("assets");

pub fn get_theme(name: &str) -> Option<String> {
    let result = ASSETS.get_file(format!("themes/{name}"));
    result.map(|f| f.contents_utf8().unwrap_or_default().to_string())
}

pub fn get_language(name: &str) -> Option<String> {
    ASSETS
        .get_file(format!("languages/{name}.json"))
        .map(|f| f.contents_utf8().unwrap_or_default().to_string())
}

pub fn get_ascii(name: &str) -> Option<String> {
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

#[cfg(debug_assertions)]
fn load_ascii_from_disk(name: &str) -> Option<String> {
    std::fs::read_to_string(format!("assets/ascii/{}.txt", name)).ok()
}

pub fn list_themes() -> Vec<String> {
    if let Some(themes_dir) = ASSETS.get_dir("themes") {
        let mut theme_files: Vec<String> = themes_dir
            .files()
            .filter(|f| f.path().file_name().is_some())
            .filter_map(|f| {
                let name = f.path().file_name()?.to_str()?.to_string();
                if name != ".gitkeep" {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        theme_files.sort_by_key(|a| a.to_lowercase());
        theme_files
    } else {
        vec![DEFAULT_THEME.to_string()]
    }
}

pub fn list_languages() -> Vec<String> {
    ASSETS
        .get_dir("languages")
        .map(|dir| {
            dir.files()
                .filter(|f| f.path().extension().is_some_and(|ext| ext == "json"))
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

#[cfg(debug_assertions)]
fn list_ascii_from_disk() -> Vec<String> {
    std::fs::read_dir("assets/ascii")
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
