# tbeep

`tbeep` is a simple command-line timer utility written in C. When the timer expires, it repeatedly plays a beep sound (WAV file) until interrupted by the user.

## Features
- Set a timer for a specified number of seconds
- Plays a beep sound when the timer ends
- Continues beeping until you stop it with `Ctrl+C`

## Requirements
- `GCC` or another `C` compiler
- `aplay` utility (usually provided by ALSA on Linux)
- Standard C library
- Unix-like environment (uses `/tmp` and `unistd.h`)

## Building

To build `tbeep`, simply run:

```sh
make
```

## Usage

```sh
./tbeep <seconds>
```

- `<seconds>`: Number of seconds to wait before beeping.

Example:

```sh
./tbeep 10
```

This sets a timer for 10 seconds. When the time is up, a beep sound will play repeatedly until you press Ctrl+C.

## How it works
- The program creates a temporary WAV file containing a beep sound.
- It waits for the specified number of seconds.
- When the timer expires, it repeatedly plays the beep sound using `aplay` until interrupted.
- The temporary WAV file is deleted when the program exits.
