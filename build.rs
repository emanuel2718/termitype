use std::{fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/themes");
    println!("cargo:rerun-if-changed=assets/languages");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Themes
    let themes_dir = Path::new(&out_dir).join("themes");
    fs::create_dir_all(&themes_dir)?;

    let assets_themes = Path::new("assets").join("themes");
    if assets_themes.exists() {
        for entry in fs::read_dir(assets_themes)? {
            let entry = entry?;
            if entry.path().is_file() {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                // Skip .gitkeep and any other non-theme files
                if file_name_str != ".gitkeep" {
                    fs::copy(entry.path(), themes_dir.join(file_name))?;
                }
            }
        }
        println!("Themes copied successfully");
    }

    // Languages
    let languages_dir = Path::new(&out_dir).join("languages");
    fs::create_dir_all(&languages_dir)?;

    let assets_languages = Path::new("assets").join("languages");
    if assets_languages.exists() {
        for entry in fs::read_dir(assets_languages)? {
            let entry = entry?;
            if entry.path().is_file() {
                fs::copy(entry.path(), languages_dir.join(entry.file_name()))?;
            }
        }
        println!("Languages copied successfully");
    }

    println!("cargo:rustc-env=THEMES_DIR={}", themes_dir.display());
    println!("cargo:rustc-env=LANGUAGES_DIR={}", languages_dir.display());

    Ok(())
}
