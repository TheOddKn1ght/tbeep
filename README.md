# tbeep

`tbeep` is a simple command-line timer utility written in C. When the timer expires, it repeatedly plays a beep sound (WAV file) until interrupted by the user.

## Features
- Set a timer for a specified duration (supports hours, minutes, seconds)
- Plays a beep sound when the timer ends
- Continues beeping until you stop it with `Ctrl+C`
- Optional countdown timer display

## Requirements
- `GCC` or another `C` compiler
- `aplay` utility (usually provided by ALSA on Linux)
- Standard C library
- **Unix-like environment only** (uses `/tmp`, `unistd.h`, and `aplay`)

## Building

To build `tbeep`, simply run:

```sh
make
```

## Usage

```sh
./tbeep [-v] <duration>
```

- `-v`: Show countdown timer (optional)
- `<duration>`: Time to wait before beeping. You can use:
  - Just seconds (e.g., `10`)
  - Or a combination: `[Nh][Nm][Ns]` (e.g., `1h30m10s`, `5m`, `90s`)

### Examples

Set a timer for 10 seconds:
```sh
./tbeep 10
```

Set a timer for 1 hour, 5 minutes, and 10 seconds, with countdown:
```sh
./tbeep -v 1h5m10s
```

Show help:
```sh
./tbeep -h
```

## How it works
- The program creates a temporary WAV file containing a beep sound in `/tmp`.
- It waits for the specified duration.
- When the timer expires, it repeatedly plays the beep sound using `aplay` until interrupted.
- The temporary WAV file is deleted when the program exits.
