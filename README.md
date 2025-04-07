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

<img align="center" alt="image" src="https://github.com/user-attachments/assets/747ecfd1-c664-4962-8049-6fa7433783a2" alt="Termitype Image" />


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
cargo install termitype@0.0.1-alpha.18
```

### TODO

- [ ] User defined config in `$XDG_CONFIG_HOME/termitype/config`. Takes precedence over default config and persitent config state in db
- [ ] Settings persistance
- [ ] Results (locally) tracking with sqlite (can be opted-out with `termitype --no-track`)
- [ ] Layout cleanup pass
- [ ] Release on:
    - [x] crates.io
    - [ ] Homebrew
    - [ ] AUR
    - [ ] nixpkgs


### Done
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
