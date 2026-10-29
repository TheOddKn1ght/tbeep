# tbeep

Terminal timer with a built-in alarm for macOS, Linux, and Windows.

Requires stable Rust. Linux builds also need ALSA development headers and pkg-config (`libasound2-dev pkg-config` on Debian/Ubuntu).

```sh
cargo install --path . --locked
tbeep            # interactive duration entry
tbeep 1h30m10s
tbeep --plain 5m
tbeep --help
```

Space pauses, `r` restarts, `m` toggles sound, and `q`/Esc/Ctrl+C quits. After expiry, Enter dismisses the repeating alarm and returns to duration entry.

[MIT license](LICENSE).
