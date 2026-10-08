# tbeep

A Rust timer with a terminal dashboard and a
built-in alarm for **macOS, Linux, and Windows**.

## Build and install

Install the current stable [Rust toolchain](https://rustup.rs/), then:

```sh
cargo build --locked --release
cargo install --path . --locked
```

The built executable is `target/release/tbeep` (`tbeep.exe` on Windows).
`make` also builds the release executable where Make is available.

On Linux, install ALSA development files and pkg-config first:

```sh
# Debian / Ubuntu
sudo apt-get install libasound2-dev pkg-config
# Fedora
sudo dnf install alsa-lib-devel pkgconf-pkg-config
```

No external audio player or temporary WAV file is needed. The original alarm is
embedded in the executable and plays through the default audio device.

## Use

```sh
tbeep                 # Enter a duration or choose a 5-, 15-, or 25-minute preset
tbeep 10              # Start a ten-second timer in the dashboard
tbeep 1h30m10s         # Hours, minutes, seconds
tbeep --plain 5m      # Plain output; alarm repeats until Ctrl+C
tbeep -v 90s          # Plain output with countdown
tbeep --help
tbeep --version
```

Durations are positive whole seconds or ordered `h`, `m`, `s` components. Each
component needs a number; repeated units, fractional values, trailing bare
numbers in mixed input, and overflowing values are rejected.

The dashboard uses large countdown digits, a progress bar, cyan accents, and
layouts that adjust to terminal size. In the entry screen, use Left/Right or Tab
to select a preset, type a duration to override it, and press Enter to start.

| Key | Action |
| --- | --- |
| Space | Pause / resume |
| `r` | Restart the original duration |
| `m` | Toggle sound (in entry, available before typing) |
| Enter after expiry | Stop the alarm and return to duration entry |
| `q`, Esc, Ctrl+C | Stop and exit |

The alarm repeats with a 500 ms gap until dismissed. Muting immediately stops
playback; unmuting an expired timer starts it again. If audio cannot initialize,
a visible warning appears and a terminal bell repeats instead. Terminal settings
may silence the bell. The terminal is restored on exit, errors, and panics.

Plain mode is selected automatically if stdin or stdout is not a terminal; it
requires a duration. `-v` also selects plain mode, retaining the original countdown
option. Redirected countdown output uses lines rather than cursor control codes.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

Tests cover parsing, timer transitions, alarm scheduling and cancellation, the
embedded WAV, keyboard controls, and rendering at different terminal sizes.
CI runs checks, tests, and release builds natively on macOS, Ubuntu, and Windows.
Audio hardware and terminal interaction still need manual checks on each platform.

MIT licensed; see [LICENSE](LICENSE).
