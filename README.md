<h1>
<p align="center">
  TermiType
</h1>
  <p align="center">
    Just another rusty TUI typing game
    <br />
    <a href="#todo">TODO</a>
    ·
    <a href="#notes">Notes</a>
  </p>
</p>



### TODO

- [ ] Change cursor style via cli
- [ ] Change cursor style at runtime
- [ ] Theme swap at runtime (Menu -> Themes -> <list of themes>)
- [ ] Theme preview in menu
- [ ] Proper Results screen
- [ ] Improve the Footer with icons if possible
- [ ] Change mode/value with the menu
- [ ] Use [tui-big-text](https://docs.rs/tui-big-text/latest/tui_big_text/) for the title
- [ ] Build Github CI
- [ ] Add more languages and word lists
- [ ] Fetch themes at build time from [iterm2Themes url](https://github.com/mbadolato/iTerm2-Color-Schemes/archive/0e23daf59234fc892cba949562d7bf69204594bb.tar.gz)
- [ ] Performance optimization pass before release (cargo flamegraph --root)

### Notes
- how to fetch themes
```sh
# hash: 12204fc99743d8232e691ac22e058519bfc6ea92de4a11c6dba59b117531c847cd6a
wget -qO- https://github.com/mbadolato/iTerm2-Color-Schemes/archive/0e23daf59234fc892cba949562d7bf69204594bb.tar.gz | tar -xvzf -
mv iTerm2-Color-Schemes-0e23daf59234fc892cba949562d7bf69204594bb/ghostty/* . && rm -rf iTerm2-Color-Schemes-0e23daf59234fc892cba949562d7bf69204594bb/
```