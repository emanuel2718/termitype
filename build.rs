use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};
use tar::Archive;

const THEME_URL: &str = "https://github.com/mbadolato/iTerm2-Color-Schemes/archive/0e23daf59234fc892cba949562d7bf69204594bb.tar.gz";
const THEME_PATH: &str = "iTerm2-Color-Schemes-0e23daf59234fc892cba949562d7bf69204594bb/ghostty";
const EXPECTED_HASH: &str = "2acc06085a83d5338192f6aab517053a6f7a4db22a86a5043a0d6ed727bf7f4b";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/themes");
    println!("cargo:rerun-if-changed=assets/languages");

    let themes_dir = Path::new("assets/themes");
    fs::create_dir_all(themes_dir)?;

    let has_themes = themes_dir.read_dir()?.filter_map(Result::ok).any(|entry| {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        entry.path().is_file() && name_str != ".gitkeep" && name_str != "tokyonight"
    });

    if !has_themes {
        println!("Downloading themes...");
        let response = ureq::get(THEME_URL).call()?;
        let mut buffer = Vec::new();
        response.into_reader().read_to_end(&mut buffer)?;

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

    Ok(())
}
