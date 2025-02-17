<h1>
<p align="center">
  termitype
</h1>
  <p align="center">
    Just another rusty TUI typing game
    <br />
    <a href="#todo">TODO</a>
    ·
    <a href="#notes">Notes</a>
  </p>
</p>

![image](https://github.com/user-attachments/assets/50764473-b647-4e7c-b07f-8dd779c1458c)



### TODO

- [ ] Proper Results screen
- [ ] User defined config in `$XDG_CONFIG_HOME/termitype/config`. Takes precedence over default config and persitent config state in db
- [ ] Settings persistance
- [ ] Results tracking with sqlite (can be opted-out with `termitype --no-track`)
- [ ] Add click actions everywhere it makes sense
- [ ] Improve the Footer with icons if possible
- [ ] Use [tui-big-text](https://docs.rs/tui-big-text/latest/tui_big_text/) for the title
- [ ] Fetch themes at build time from [iterm2Themes url](https://github.com/mbadolato/iTerm2-Color-Schemes/archive/0e23daf59234fc892cba949562d7bf69204594bb.tar.gz)
- [ ] Performance optimization pass before release (cargo flamegraph --root)

### Done
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



### Notes
- how to fetch themes
```sh
# hash: 12204fc99743d8232e691ac22e058519bfc6ea92de4a11c6dba59b117531c847cd6a
wget -qO- https://github.com/mbadolato/iTerm2-Color-Schemes/archive/0e23daf59234fc892cba949562d7bf69204594bb.tar.gz | tar -xvzf -
mv iTerm2-Color-Schemes-0e23daf59234fc892cba949562d7bf69204594bb/ghostty/* assets/themes/ && rm -rf iTerm2-Color-Schemes-0e23daf59234fc892cba949562d7bf69204594bb/
```
