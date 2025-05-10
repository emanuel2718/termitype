<h1>
<p align="center">
  termitype
</h1>
  <p align="center">
    Just another rusty TUI typing game
    <br />
    <a href="#development">Development</a>
    ·
    <a href="#Installation">Installation</a>
  </p>
</p>

<img align="center" alt="image" src="https://github.com/user-attachments/assets/ed30fa72-7d82-4f50-b313-8f24e0705fca" alt="Termitype Image" />

## Development

### Getting Started

1. Clone the repository:

```sh
git clone https://github.com/emanuel2718/termitype.git
cd termitype
```

2. Optional: Install theme Pack

```sh
# NOTE: This is only required for local development. In production, this is handled automatically by the build process.
./scripts/install-themes.sh
```

3. Build and run:

```sh
# Normal run
cargo run

# Debug mode
cargo run -- --debug
```

## Installation

```sh
cargo install termitype@0.0.1-alpha.27
```

### TODO

- [ ] Locally track stats (results) with sqlite (can be opted-out with `termitype --no-track`)
- [ ] Release on:
  - [x] crates.io
  - [ ] Homebrew
  - [ ] AUR
  - [ ] nixpkgs

### Done

- [x] Layout cleanup pass
- [x] Settings persistance
- [x] Proper Results screen
- [x] Build Github CI
- [x] Theme swap at runtime (Menu -> Themes -> <list of themes>)
- [x] Theme preview in menu
- [x] Change cursor style via cli
- [x] Change cursor style at runtime
- [x] Only show (3) lines with scroll off
- [x] underline wrong words (must have at least one wrong character for it to be underlined)
- [x] Add scrolbar to the menu
- [x] Preview the cusror style change like we do with the theme picker
- [x] Change language at runtime
- [x] Add more languages and word lists (good enough for now)
- [x] Change mode/value with the menu
- [x] Fetch themes at build time from [iterm2Themes url](https://github.com/mbadolato/iTerm2-Color-Schemes/archive/0e23daf59234fc892cba949562d7bf69204594bb.tar.gz)
- [x] Don't include `debug` flag in release
- [x] Add LICENSE
