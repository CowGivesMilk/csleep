# csleep

A tiny command-line utility that sleeps in chunks for a specified amount of time while displaying a smooth progress bar with the remaining time.

Unlike the shell's built-in `sleep`, `csleep` gives you visual feedback while it waits.

## Features

- Accept integer or floating-point durations (`90`, `1.5`, `0.25`)
- Time unit suffixes: `s` (seconds), `m` (minutes), `h` (hours), `d` (days)
- Compound durations in a single argument, e.g. `2h30m`
- Multiple arguments summed together, e.g. `csleep 1h 30m 15s`
- Smooth 40-cell progress bar using eighth-block characters (`▏▎▍▌▋▊▉█`)
- Live percentage and remaining-time countdown (`HH:MM:SS`)
- Progress bar is written to **stderr**, so stdout stays clean for piping
- Small, dependency-free, single-binary

## Installation

### From source

```sh
git clone https://github.com/CowGivesMilk/csleep
cd csleep
cargo build --release
