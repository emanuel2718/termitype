pub use crate::assets::{get_ascii, list_ascii};
use crate::constants::DEFAULT_ASCII_ART;
use std::str::FromStr;

// TODO: add a way to allow custom art loading from `$XDG_CONFIG_HOME/termitype/ascii/{art}.txt`

#[derive(Debug, Clone)]
pub struct Ascii {
    pub name: String,
}

pub fn get_default_art_by_os() -> &'static str {
    // NOTE: this only gets called the first time app gets launched
    match () {
        _ if cfg!(target_os = "macos") => "Apple",
        _ if cfg!(target_os = "windows") => "Windows7",
        _ if cfg!(target_os = "linux") => get_art_by_linux_distro(),
        _ => DEFAULT_ASCII_ART,
    }
}

/// Detects the Linux distribution and returns the appropriate ASCII art name.
/// Falls back to `DEFAULT_ASCII_ART` if detection fails or no matching art is found.
fn get_art_by_linux_distro() -> &'static str {
    // NOTE: we could use `https://crates.io/crates/os_info` but not needed for now
    let os_release = match std::fs::read_to_string("/etc/os-release") {
        Ok(content) => content,
        Err(_) => return DEFAULT_ASCII_ART,
    };

    // Reference: https://github.com/which-distro/os-release
    for line in os_release.lines() {
        if let Some(id) = line.strip_prefix("ID=") {
            let id = id.trim_matches('"').trim();
            return match id {
                "arch" if is_potentially_omarchy() => "Omarchy",
                "arch" => "Arch Linux",
                "manjaro" => "Manjaro Linux",
                "ubuntu" => "Ubuntu",
                "debian" => "Debian Linux",
                "fedora" => "Fedora Linux",
                "gentoo" => "Gentoo Linux",
                "void" => "Void Linux",
                "nixos" => "NixOS",
                "kali" => "Kali Linux",
                "linuxmint" => "Linux Mint",
                "omarchy" => "Omarchy",
                _ => "GNU",
            };
        }
    }

    DEFAULT_ASCII_ART
}

fn is_potentially_omarchy() -> bool {
    // check if the env var is set
    if let Ok(path) = std::env::var("OMARCHY_PATH") {
        if std::path::Path::new(&path).exists() {
            return true;
        }
    }
    // manual check for the manual path
    // ref: https://github.com/basecamp/omarchy/blob/master/install.sh
    if std::path::Path::new(&format!(
        "{}/.local/share/omarchy",
        std::env::var("HOME").unwrap_or_default()
    ))
    .exists()
    {
        return true;
    }
    // lastly manually check for the omarchy version. not the best but maybe one day
    // omarchy will include the change of `/etc/os-release` to have id=omarchy
    std::process::Command::new("omarchy-version")
        .output()
        .is_ok()
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
