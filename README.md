<div align="center">

# 🦀 termitype

[![Crates.io](https://img.shields.io/crates/v/termitype.svg)](https://crates.io/crates/termitype)
[![Build Status](https://github.com/emanuel2718/termitype/workflows/CI/badge.svg)](https://github.com/emanuel2718/termitype/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

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

```sh
# TODO: link to github releases
```

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

```
Usage: termitype [OPTIONS]

Options:
  -l, --language <LANGUAGE>        Language dictionary to use
  -t, --time <SECONDS>             Test duration in seconds
  -w, --words <"WORD1 WORD2 ...">  Custom words for the test
      --word-count <COUNT>         Number of words to type
  -T, --theme <THEME_NAME>         Color theme to use
      --ascii <ART_NAME>           ASCII art for results screen
      --picker <STYLE>             Menu style [possible values: quake, telescope, ivy, minimal]
      --list-themes                List all available themes
      --list-languages             List all available languages
      --list-ascii                 List all available ASCII arts
  -s, --use-symbols                Include symbols in test words
  -p, --use-punctuation            Include punctuation in test words
  -n, --use-numbers                Include numbers in test words
      --color-mode <MODE>          Color support [possible values: basic, extended, truecolor]
      --cursor-style <STYLE>       Cursor style [possible values: beam, block, underline, blinking-beam, blinking-block, blinking-underline]
      --lines <COUNT>              Number of visible text lines [default: 3]
  -d, --debug                      Enable debug mode
      --show-fps                   Display FPS counter
      --hide-live-wpm              Hide live WPM counter
      --hide-cursorline            Hide menu cursor highlight
      --monochromatic-results      Use simplified results colors
  -h, --help                       Print help
  -V, --version                    Print version

EXAMPLES:
  termitype -t 60                        Run a 60-second typing test
  termitype --word-count 100             Test will contain exactly 100 words
  termitype -T "catppuccin-mocha"        Use cattpuccin-mocha theme
  termitype -l spanish                   Use spanish test words
  termitype -spn                         Enable symbols, punctuation, and numbers
  termitype --list-themes                Show all available themes
  termitype --picker telescope           Use floating menu style
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

2. **Optional: Install theme pack** (development only):

```sh
./scripts/install-themes.sh
```

_Note: In production builds, themes are automatically fetched and included._

3. **Run the application**:

```sh
# Development build
cargo run

# Release build
cargo run --release

# With debug logging
cargo run -- --debug

# Tail logs with something like this (MacOS example)
tail -f ~/Library/Application\ Support/termitype/debug.log

# Tail logs with something like this (Linux example)
tail -f ~/.config/termitype/debug.log
```

## Themes

Termitype includes a curated collection of themes sourced from the [iTerm2 Ghostty Color Schemes Repo](https://github.com/mbadolato/iTerm2-Color-Schemes/tree/master/ghostty) repository. Themes are automatically fetched during the build process and can be previewed and changed in real-time.

## Contributing

> [!Warning]
> TODO: write out the contribution guideline just in the case there's one person interested in this.

## Roadmap

### Upcoming Features

- [ ] **Package Distribution**: Release on Homebrew, AUR, nixpkgs, etc.
- [ ] **User config file**: Have a user editable config file in `$XDG_CONFIG_HOME/termitype/config.toml`
- [ ] **Custom ascii arts**: Allow usage of custom ascii arts
- [ ] **Custom theme**: Allow setting custom themes with names
- [ ] **Advanced Analytics**: Detailed typing pattern analysis and options for graph mode results
- [ ] **Local Statistics Tracking**: SQLite-based stats with opt-out option

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) for the amazing TUI framework.
- [Monketype](https://github.com/monkeytypegame/monkeytype) for the inspiration.
- [iTerm2 Ghostty Color Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes/tree/master/ghostty) for the themes.
