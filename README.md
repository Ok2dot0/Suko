# Suko

A multi-frontend Sudoku solver in Rust with both an algorithmic (backtracking) solver and a human-oriented logical solver. Includes:

- Core library (`suko-core`) with board parsing/validation, step tracing, and solver traits.
- TUI (`suko-tui`) using ratatui + crossterm for interactive terminal solving.
- GUI (`suko-gui`) using egui/eframe for a desktop window.

## Quick start

- TUI: run `cargo run -p suko-tui` and paste an 81-char puzzle string (use '.' or '0' for blanks), press `i` to load, `b` for backtracking steps, `l` for one logical step, `n` to apply next step.
- GUI: run `cargo run -p suko-gui` and use the input field and buttons.

## Devlogs

Development logs are written to `devlogs/devlogN.txt` as features land. You can commit between milestones.

## License

MIT
