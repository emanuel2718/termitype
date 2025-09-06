use crate::constants::DEFAULT_THEME;
use include_dir::{include_dir, Dir};

pub static ASSETS: Dir = include_dir!("assets");

pub fn get_theme(name: &str) -> Option<String> {
    let result = ASSETS.get_file(format!("themes/{name}"));
    result.map(|f| f.contents_utf8().unwrap_or_default().to_string())
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
