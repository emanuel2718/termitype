<div align="center">

# 🦀 termitype

[![Crates.io](https://img.shields.io/crates/v/termitype.svg)](https://crates.io/crates/termitype)
[![Build Status](https://github.com/emanuel2718/termitype/workflows/CI/badge.svg)](https://github.com/emanuel2718/termitype/actions)
[![License: GPL-3](https://img.shields.io/badge/License-GPL3-blue.svg)](https://opensource.org/license/GPL-3.0)

**Feature-rich terminal typing test**

<p align="center">
    Heavily inspired by a certain typing test you might know.
    <br />
    <a href="#installation">Installation</a>
    ·
    <a href="#usage">Usage</a>
    ·
    <a href="#options">Options</a>
    ·
    <a href="#development">Development</a>
    ·
    <a href="#contributing">Contributing</a>
    ·
    <a href="#roadmap">Roadmap</a>
    ·
    <a href="#acknowledgments">Acknowledgments</a>
  </p>
</p>
</div>

<br />
<p align="center">
  <img src="https://raw.githubusercontent.com/emanuel2718/termitype/main/assets/demo.gif" alt="Termitype demo" width="600">
</p>

## Installation

### From Crates.io

```sh
cargo install termitype
```

### From Source

```sh
cargo install --git https://github.com/emanuel2718/termitype.git termitype
```

### From Released Binaries

**📦 Download prebuilt binaries from the [latest release](https://github.com/emanuel2718/termitype/releases)**

Pre-compiled binaries are available for the following platforms:

| Platform    | Architecture  | Download                                                                                                                               |
| ----------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| **Linux**   | x86_64 (gnu)  | [Download Latest](https://github.com/emanuel2718/termitype/releases/download/v0.0.6/termitype-v0.0.6-x86_64-unknown-linux-gnu.tar.gz)  |
| **Linux**   | x86_64 (musl) | [Download Latest](https://github.com/emanuel2718/termitype/releases/download/v0.0.6/termitype-v0.0.6-x86_64-unknown-linux-musl.tar.gz) |
| **macOS**   | Intel         | [Download Latest](https://github.com/emanuel2718/termitype/releases/download/v0.0.6/termitype-v0.0.6-x86_64-apple-darwin.tar.gz)       |
| **macOS**   | Apple Silicon | [Download Latest](https://github.com/emanuel2718/termitype/releases/download/v0.0.6/termitype-v0.0.6-aarch64-apple-darwin.tar.gz)      |
| **Windows** | x86_64        | [Download Latest](https://github.com/emanuel2718/termitype/releases/download/v0.0.6/termitype-v0.0.6-x86_64-pc-windows-msvc.zip)       |

### From Package Manager

<details>
<summary>🚧 Coming Soon 🚧</summary>

- **Homebrew**: `brew install termitype` _(planned)_
- **AUR**: `yay -S termitype` _(planned)_
- **Nix**: `nix-shell -p termitype` _(planned)_
- **Windows**: `scoop install termitype` _(?)_

</details>

## Usage

### Basic Usage

```sh
# Start typing
termitype

# See available CLI arg options (all options can also be configured via the in-game menu)
termitype --help
```

## Options

| Option                          | Description                                                                                          |
| :------------------------------ | :--------------------------------------------------------------------------------------------------- |
| `-l`, `--language <LANG>`       | Language dictionary to use                                                                           |
| `-t`, `--time <SECONDS>`        | Test duration in seconds                                                                             |
| `-w`, `--words <"WORD1 WORD2">` | Custom words for the test                                                                            |
| `--word-count <COUNT>`          | Number of words to type                                                                              |
| `-s`, `--use-symbols`           | Include symbols in test words                                                                        |
| `-p`, `--use-punctuation`       | Include punctuation in test words                                                                    |
| `-n`, `--use-numbers`           | Include numbers in test words                                                                        |
| `--lines <COUNT>`               | Number of visible text lines (default: 3)                                                            |
| `-T`, `--theme <THEME>`         | Theme to use                                                                                         |
| `--ascii <ART>`                 | ASCII art for results screen                                                                         |
| `--picker-style <STYLE>`        | Menu style (`quake`, `telescope`, `ivy`, `minimal`)                                                  |
| `--results-style <STYLE>`       | Results display style (`graph`, `minimal`, `neofetch`)                                               |
| `--cursor-style <STYLE>`        | Cursor style (`beam`, `block`, `underline`, `blinking-beam`, `blinking-block`, `blinking-underline`) |
| `--show-fps`                    | Display FPS counter                                                                                  |
| `--hide-live-wpm`               | Hide live WPM counter                                                                                |
| `--hide-cursorline`             | Hide menu cursor highlight                                                                           |
| `--hide-notifications`          | Hide notifications                                                                                   |
| `--monochromatic-results`       | Use simplified results colors                                                                        |
| `--list-themes`                 | List all available themes                                                                            |
| `--list-languages`              | List all available languages                                                                         |
| `--list-ascii`                  | List all available ASCII arts                                                                        |
| `--color-mode <MODE>`           | Color support (`basic`, `extended`, `truecolor`)                                                     |
| `--no-track`                    | Do not locally track tests results nor stats                                                         |
| `--reset-db`                    | Reset and clears the content of the local database                                                   |
| `-d`, `--debug`                 | Enable debug mode                                                                                    |
| `-h`, `--help`                  | Print help                                                                                           |
| `-V`, `--version`               | Print version                                                                                        |

### Examples

```sh
# All of the options below can also be changed at runtime via the menu.
termitype -t 60                        # Run a 60-second typing test
termitype --word-count 100             # Test will contain exactly 100 words
termitype -T "catppuccin-mocha"        # Use catppuccin-mocha theme
termitype -l spanish                   # Use Spanish test words
termitype -spn                         # Enable symbols, punctuation, and numbers
termitype --list-themes                # Show all available themes
termitype --results-style neofetch     # Use neofetch inspired results
termitype --picker-style telescope     # Use floating menu style
termitype --no-track                   # Do not locally track test results nor stats
termitype --hide-notifications         # Do not show notifications
```

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Quick Start

1. **Clone the repository**:

```sh
git clone https://github.com/emanuel2718/termitype.git
cd termitype
```

2. **Run the application**:

```sh
# Development build
cargo run

# Release build
cargo run --release

# With debug logging
cargo run -- --debug

# Tail logs with something like this (MacOS example)
tail -f ~/Library/Application\ Support/termitype/termitype.log

# Tail logs with something like this (Linux example)
tail -f ~/.config/termitype/termitype.log
```

## Themes

Termitype includes a curated collection of themes sourced from the [iTerm2 Ghostty Color Schemes Repo](https://github.com/mbadolato/iTerm2-Color-Schemes/tree/master/ghostty) repository. Themes can be previewed and changed in real-time.

## Contributing

> [!Warning]
> TODO: write out the contribution guideline just in the case there's one person interested in this.

## Roadmap

### Upcoming Features

- [ ] **Package Distribution**: Release on Homebrew, AUR, nixpkgs, etc.
- [ ] **User config file**: Have a user editable config file in `$XDG_CONFIG_HOME/termitype/config.toml`
- [ ] **Custom ascii arts**: Allow usage of custom ascii arts
- [ ] **Custom theme**: Allow setting custom themes with names
- [ ] **Wordlist Improvements**: Improve the quality and distribution of words
- [ ] **Multiplayer**: Race other people in realtime with private rooms of sort (will use websockets for this)
- [x] **Local Results Tracking**: Track test results over time (best use case is to track highest WPM on specific modes) with opt-out option

## License

This project is licensed under the GPL-3.0 license - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) for the amazing TUI framework.
- [Monketype](https://github.com/monkeytypegame/monkeytype) for the inspiration.
- [iTerm2 Ghostty Color Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes/tree/master/ghostty) for the themes.
