# Goobler

Desktop noise generator. Picks a noise color, applies some filters, plays it through your speakers. Goes fullscreen with a black screen so you can sleep.

## Build

```
cargo build --release
```

## Run

```
cargo run --release
```

## Controls

| Key | Action |
|---|---|
| Escape | Save settings, go back to noise |
| Up / Down | Navigate settings |
| Left / Right | Adjust the selected value |
| Enter | Toggle or cycle the selected item |
| Mouse | Sliders, checkboxes, dropdown, buttons |

## Noise types

White, Pink, Brown, Green, Blue, Violet. Each noise color has a different frequency distribution — Pink and Brown are good for sleep, White is harsher, the others are more specialized.

## Audio pipeline

```
Noise gen → high-pass → low-pass → bass boost → volume → output
```

Smoothing and a noise overlay can also be mixed in. All processing runs per-sample at the configured sample rate (44100 or 48000 Hz).

## Config

Saved to `~/.config/goobler/config.json` on exit if there are unsaved changes. You can choose "Exit Without Saving" to discard edits and reload from disk.

## Tech

- Rust, egui/eframe for the UI
- rodio/cpal for audio output
- Biquad filters (Butterworth) for low-pass, high-pass, and bass boost
- Paul Kellet's method for pink noise
