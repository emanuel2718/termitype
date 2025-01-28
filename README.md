# Termitype


### TODO

- [ ] only have three lines or less of text showing at a time. They should scrolloff when the
user gets to a new line.
- [x] tracker for all the metrics
  - wpm (normal/raw)
  - accuracy
  - consistency
  - time tracking
  - wrong/correct chars/words
  - wrong/total keystrokes
- [x] toggle mode/punctuation/numbers/symbols at runtime
- [x] click actions
- [x] menu
- [ ] themes
  - [x] `termitype --list-themes`
  - [x] `termitype --theme tokyonight`
  - [ ] custom themes in `$XDG_CONFIG_HOME/termitype/theme/<custom-theme>`
  - [ ] swap themes at runtime (Menu -> Themes -> <list of themes>) with preview
  - [x] use iterm/ghostty color pattern ?
  - [ ] fetch themes at build time from https://github.com/mbadolato/iTerm2-Color-Schemes
  -  [iterm2Themes url](https://github.com/mbadolato/iTerm2-Color-Schemes/archive/0e23daf59234fc892cba949562d7bf69204594bb.tar.gz)

  ### how to fetch themes
  ```sh
  # hash: 12204fc99743d8232e691ac22e058519bfc6ea92de4a11c6dba59b117531c847cd6a
  wget -qO- https://github.com/mbadolato/iTerm2-Color-Schemes/archive/0e23daf59234fc892cba949562d7bf69204594bb.tar.gz | tar -xvzf -
  mv iTerm2-Color-Schemes-0e23daf59234fc892cba949562d7bf69204594bb/ghostty/* . && rm -rf iTerm2-Color-Schemes-0e23daf59234fc892cba949562d7bf69204594bb/
  ```





- [x] implement word pool generator using languages as json files

### Notes
- think about using json for the languages like monkeytype does [monkeytype _list.json](https://github.com/monkeytypegame/monkeytype/blob/bb3a99861fe96a7ecf4a31758f87c3b8057c6e29/frontend/static/languages/_list.json)

