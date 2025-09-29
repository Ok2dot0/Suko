# Suko

A multi-frontend Sudoku solver in Rust with both an algorithmic (backtracking) solver and a human-oriented logical solver. Includes:

- Core library (`suko-core`) with board parsing/validation, step tracing, and solver traits.
- TUI (`suko-tui`) using ratatui + crossterm for interactive terminal solving.
- GUI (`suko-gui`) using egui/eframe for a desktop window.

## Quick start

- TUI: run `cargo run -p suko-tui` and paste an 81-char puzzle string (use '.' or '0' for blanks), press `i` to load, `b` for backtracking steps, `l` for one logical step, `n` to apply next step.
- GUI: run `cargo run -p suko-gui` and use the input field and buttons.

Session exporting:
- TUI: press `s` to export the current steps to `logs/sessions/session_*.md`.
- GUI: click "Export session" to write the markdown log to `logs/sessions`.

Examples:
- Example puzzles are in `examples/`. You can open them in a text editor and paste into TUI/GUI input, or type directly into the grid.
	- `examples/easy1.sdk`
	- `examples/hard1.sdk`

Visualization hints:
- TUI: peers of the selected cell are dimmed; selected cell is highlighted; candidates for the selected cell are shown in the Help panel.
- GUI: peers are shaded; fixed values are colored; last step cell can be emphasized (toggle).

## Build, Run, and Ship

- Build (debug):
	- `cargo build`
- Run:
	- GUI: `cargo run -p suko-gui`
	- TUI: `cargo run -p suko-tui`
- Release binaries:
	- `cargo build --release`
	- Binaries will appear in `target/release/`.

The workspace uses edition 2021 and resolver v2. A release profile with LTO and strip is configured for small, fast binaries.

## Devlogs

Development logs live in `devlogs/devlogN.txt`. They’re written like a short journal about what changed and why, not a checklist.

## License

MIT
