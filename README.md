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

## Usage

### Basic Usage

```sh
# Start typing
termitype

# See available CLI arg options (all options can also be configured via the in-game menu)
termitype --help
```

## Options

| Option                          | Description                                                                                     |
| :------------------------------ | :---------------------------------------------------------------------------------------------- |
| `-t`, `--time <SECONDS>`        | Test duration in seconds. Enforces Time mode                                                    |
| `-w`, `--words <"WORD1 WORD2">` | Custom words for the test. Enforces Word mode                                                   |
| `-c`, `--count <COUNT>`         | Number (count) of words to type                                                                 |
| `-n`, `--use-numbers`           | Include numbers in the test word pool                                                           |
| `-s`, `--use-symbols`           | Include symbols in the test word pool                                                           |
| `-p`, `--use-punctuation`       | Include punctuation in the test word pool                                                       |
| `-l`, `--language <LANG>`       | Language dictionary the test will use                                                           |
| `--theme <THEME>`               | The theme of the application                                                                    |
| `--ascii <ASCII>`               | The ASCII art used in the `Neofetch` results                                                    |
| `--cursor <STYLE>`              | Cursor style variant: beam, block, underline, blinking-beam, blinking-block, blinking-underline |
| `--results <STYLE>`             | Results style variant: minimal, neofetch, graph                                                 |
| `--lines <COUNT>`               | Number of visible text lines [default: 3]                                                       |
| `--hide-live-wpm`               | Hide live WPM counter                                                                           |
| `--hide-notifications`          | Hide notifications                                                                              |
| `--no-track`                    | Do not locally track test results                                                               |

### Examples

```sh
# All of the options below can also be changed at runtime via the menu.
termitype -t 60                        # Run a 60-second typing test
termitype -c 100                       # Test will contain exactly 100 random words
termitype -T "catppuccin-mocha"        # Use catppuccin-mocha theme
termitype -l spanish                   # Use Spanish test words
termitype -spn                         # Enable symbols, punctuation, and numbers
termitype --results-style neofetch     # Use neofetch inspired results
termitype --no-track                   # Do not locally track test results nor stats
termitype --hide-notifications         # Do not show notifications
```

## Development

### Prerequisites

- Rust 1.87+
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
