use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};
use tar::Archive;

const THEME_URL: &str = "https://github.com/mbadolato/iTerm2-Color-Schemes/archive/1e4957e65005908993250f8f07be3f70e805195e.tar.gz";
const THEME_PATH: &str = "iTerm2-Color-Schemes-1e4957e65005908993250f8f07be3f70e805195e/ghostty";
const EXPECTED_HASH: &str = "c690e2b57a59add53f11c80bc86e06d1c1224f8af8daf8b2f832402e6cb6b101";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/themes");
    println!("cargo:rerun-if-changed=assets/languages");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Themes
    let themes_dir = Path::new(&out_dir).join("themes");
    fs::create_dir_all(&themes_dir)?;

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

    let has_themes = themes_dir.read_dir()?.filter_map(Result::ok).any(|entry| {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        entry.path().is_file() && name_str != ".gitkeep" && name_str != "tokyonight"
    });

    if !has_themes {
        println!("Downloading themes...");
        let response = ureq::get(THEME_URL).call()?;
        let mut buffer = Vec::new();
        response
            .into_body()
            .into_reader()
            .read_to_end(&mut buffer)?;

        // verify the hash
        let mut hasher = Sha256::new();
        hasher.update(&buffer);
        let hash = format!("{:x}", hasher.finalize());
        if hash != EXPECTED_HASH {
            return Err(format!("Hash mismatch. Expected {}, got {}", EXPECTED_HASH, hash).into());
        }

        // extract the themes
        let temp_dir = tempfile::tempdir()?;
        let theme_path = temp_dir.path().join(THEME_PATH);

        Archive::new(GzDecoder::new(&buffer[..])).unpack(temp_dir.path())?;

        if theme_path.exists() {
            for entry in fs::read_dir(theme_path)? {
                let entry = entry?;
                fs::copy(entry.path(), themes_dir.join(entry.file_name()))?;
            }
            println!("Themes installed successfully");
        }
    }

    println!("cargo:rustc-env=THEMES_DIR={}", themes_dir.display());
    println!("cargo:rustc-env=LANGUAGES_DIR={}", languages_dir.display());

    Ok(())
}
