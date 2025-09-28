use std::io;
use std::time::{Duration, Instant};
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{prelude::*, widgets::*};
use suko_core::{board::Board, solver::BacktracingBruteSolver, puzzle::PuzzleGenerator};
use std::fs;

fn draw_board(frame: &mut Frame, area: Rect, board: &Board, sel: (usize, usize)) {
    let mut lines: Vec<Line> = Vec::new();
    // Top border not drawn; the surrounding Block provides it. We'll draw row separators between 3x3 bands.
    for r in 0..9 {
        let mut spans: Vec<Span> = Vec::new();
        for c in 0..9 {
            let v = board.cells[r][c].value;
            let ch = if v == 0 { '·' } else { char::from(b'0' + v) };
            let mut style = Style::default();
            // Subgrid background hint via gray tone
            let subgrid_tint = if (r/3 + c/3) % 2 == 0 { Color::DarkGray } else { Color::Reset };
            if subgrid_tint != Color::Reset { style = style.bg(subgrid_tint); }
            // peer highlight: same row, col, or box as selected
            let in_same_row = r == sel.0;
            let in_same_col = c == sel.1;
            let in_same_box = (r/3 == sel.0/3) && (c/3 == sel.1/3);
            if in_same_row || in_same_col || in_same_box { style = style.fg(Color::Gray); }
            if (r, c) == sel { style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD); }
            if board.cells[r][c].fixed { style = style.fg(Color::Cyan); }
            spans.push(Span::styled(format!(" {} ", ch), style));
            // Box vertical separator
            if c % 3 == 2 && c != 8 { spans.push(Span::styled("┃", Style::default().fg(Color::White))); spans.push(Span::raw(" ")); }
            else { spans.push(Span::raw("")); }
        }
        lines.push(Line::from(spans));
        // Heavy horizontal separator between boxes
        if r % 3 == 2 && r != 8 {
            lines.push(Line::from(Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Style::default().fg(Color::White))));
        }
    }
    let block = Block::default().borders(Borders::ALL).title("Sudoku");
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn try_move_sel(sel: &mut (usize, usize), last_move: &mut Instant, cooldown: Duration, dr: isize, dc: isize) {
    let now = Instant::now();
    if now.duration_since(*last_move) < cooldown { return; }
    let nr = ((sel.0 as isize + dr).rem_euclid(9)) as usize;
    let nc = ((sel.1 as isize + dc).rem_euclid(9)) as usize;
    *sel = (nr, nc);
    *last_move = now;
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input_str = String::new();
    let mut brute = BacktracingBruteSolver::new();
    let mut board = Board::empty();
    let mut sel: (usize, usize) = (0, 0);
    // Edit & modes
    let mut path_edit = false; // when true, keystrokes go to input_str only
    // No maze features

    let res = run_app(&mut terminal, &mut board, &mut input_str, &mut brute, &mut sel, &mut path_edit);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res { eprintln!("Error: {err:#}"); }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, board: &mut Board, input_str: &mut String, brute: &mut BacktracingBruteSolver, sel: &mut (usize, usize), path_edit: &mut bool) -> anyhow::Result<()> {
    let cooldown = Duration::from_millis(120);
    let mut last_move = Instant::now() - cooldown;
    let mut status = String::new();
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(18),
                    Constraint::Length(6),
                    Constraint::Min(3),
                ]).split(f.size());
            draw_board(f, chunks[0], board, *sel);

            // Help/status
            let mut cand_str = String::new();
            if board.cells[sel.0][sel.1].value==0 {
                let cand = board.candidates(sel.0, sel.1);
                let mut first=true;
                for v in 1..=9 { if cand[v as usize] { if !first { cand_str.push(' '); } cand_str.push(char::from(b'0'+v)); first=false; } }
            }
            let help_text = format!(
                "arrows/hjkl=move | 1-9=set | 0/.=clear | o=Open board.sdk | s=Save board.sdk | O=Open path | S=Save path | Tab: focus input | c=Clear | b=Backtracing solve | p=Random puzzle | P=Seeded puzzle | q=Quit\nSelected: ({}, {})   Candidates: [{}]   Status: {}",
                sel.0 + 1, sel.1 + 1, cand_str, status
            );
            let title = "Help";
            let help = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title(title));
            f.render_widget(help, chunks[1]);

            // Input field
            let mut input_block = Block::default().borders(Borders::ALL);
            input_block = if *path_edit { input_block.title("Input (FOCUSED): Enter=Open/Load • Esc=cancel") } else { input_block.title("Input: Paste 81 chars or type a path; Tab=focus") };
            let input = Paragraph::new(input_str.as_str()).block(input_block);
            f.render_widget(input, chunks[2]);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(k) => {
                    // Path edit mode: capture text safely
                    if *path_edit {
                        match (k.code, k.modifiers) {
                            (KeyCode::Esc, _) => { *path_edit = false; },
                            (KeyCode::Enter, _) => {
                                // Try 81 chars first, else treat as path
                                if let Ok(norm) = super_simplify_normalize(input_str) {
                                    match Board::parse(&norm) { Ok(b) => { *board=b; *sel=(0,0); status = "Loaded from pasted text".into(); *path_edit = false; }, Err(e) => { status = format!("Parse failed: {}", e); } }
                                } else {
                                    match fs::read_to_string(input_str.trim()) {
                                        Ok(raw) => if let Ok(norm) = super_simplify_normalize(&raw) { if let Ok(b) = Board::parse(&norm) { *board=b; *sel=(0,0); status = format!("Opened {}", input_str.trim()); *path_edit = false; } } else { status = "Input lacks 81 chars".into(); },
                                        Err(e) => status = format!("Open failed: {}", e),
                                    }
                                }
                            },
                            (KeyCode::Backspace, _) => { input_str.pop(); },
                            (KeyCode::Char('s'), m) if m.contains(KeyModifiers::CONTROL) => {
                                if !input_str.is_empty() {
                                    match fs::write(input_str.trim(), board_to_sdk(board)) { Ok(_) => status = format!("Saved {}", input_str.trim()), Err(e) => status = format!("Save failed: {}", e) }
                                }
                            },
                            // Do not exit edit mode on Tab; keep focus until Enter/Esc
                            (KeyCode::Char(ch), _) => { if input_str.len() < 512 { input_str.push(ch); } },
                            _ => {}
                        }
                        continue; // skip other handlers while editing
                    }

                    // Normal mode (not editing path)
                    match k.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Tab => { *path_edit = true; },
                        KeyCode::Char('o') => {
                            if let Ok(raw) = fs::read_to_string("board.sdk") {
                                if let Ok(norm) = super_simplify_normalize(&raw) {
                                    if let Ok(b) = Board::parse(&norm) { *board = b; *sel=(0,0); }
                                }
                            }
                        },
                        KeyCode::Char('O') => {
                            if !input_str.is_empty() {
                                match fs::read_to_string(input_str.trim()) {
                                    Ok(raw) => if let Ok(norm) = super_simplify_normalize(&raw) { if let Ok(b) = Board::parse(&norm) { *board=b; *sel=(0,0); status = format!("Opened {}", input_str.trim()); } } else { status = "Input lacks 81 chars".into(); },
                                    Err(e) => status = format!("Open failed: {}", e),
                                }
                            }
                        },
                        KeyCode::Char('b') => {
                            status = "Solving…".into();
                            if let Some(solved) = brute.solve_to_completion(board) { *board = solved; status = "Solved".into(); } else { status = "No solution".into(); }
                        },
                        KeyCode::Char('p') => {
                            let mut gen = PuzzleGenerator::new(None);
                            let clues = 30;
                            *board = gen.generate_puzzle(clues);
                            *sel = (0,0);
                            status = format!("Generated puzzle with ~{} clues", clues);
                        },
                        KeyCode::Char('P') => {
                            match input_str.trim().parse::<u64>() {
                                Ok(seed) => {
                                    let mut gen = PuzzleGenerator::new(Some(seed));
                                    *board = gen.generate_puzzle(30);
                                    *sel = (0,0);
                                    status = format!("Generated seeded puzzle (seed {})", seed);
                                },
                                Err(_) => { status = "Enter numeric seed in input, then press P".into(); }
                            }
                        },
                        KeyCode::Char('c') => { *board = Board::empty(); *sel=(0,0); status = "Cleared".into(); },
                        KeyCode::Left => { try_move_sel(sel, &mut last_move, cooldown, 0, -1); },
                        KeyCode::Right => { try_move_sel(sel, &mut last_move, cooldown, 0, 1); },
                        KeyCode::Up => { try_move_sel(sel, &mut last_move, cooldown, -1, 0); },
                        KeyCode::Down => { try_move_sel(sel, &mut last_move, cooldown, 1, 0); },
                        KeyCode::Char('h') => { try_move_sel(sel, &mut last_move, cooldown, 0, -1); },
                        KeyCode::Char('l') => { try_move_sel(sel, &mut last_move, cooldown, 0, 1); },
                        KeyCode::Char('k') => { try_move_sel(sel, &mut last_move, cooldown, -1, 0); },
                        KeyCode::Char('j') => { try_move_sel(sel, &mut last_move, cooldown, 1, 0); },
                        KeyCode::Char('g') => { for r in 0..9 { for c in 0..9 { let v=board.cells[r][c].value; board.cells[r][c].fixed = v!=0; }} },
                        KeyCode::Char('u') => { for r in 0..9 { for c in 0..9 { board.cells[r][c].fixed = false; }} },
                        KeyCode::Char('.') | KeyCode::Char('0') => { if !board.cells[sel.0][sel.1].fixed { board.cells[sel.0][sel.1].value=0; } },
                        KeyCode::Char(ch) if ch.is_ascii_digit() => {
                            if ('1'..='9').contains(&ch) && !board.cells[sel.0][sel.1].fixed {
                                board.cells[sel.0][sel.1].value = ch.to_digit(10).unwrap() as u8;
                            }
                        },
                        KeyCode::Char('s') => { let _ = fs::write("board.sdk", board_to_sdk(board)); status = "Saved to board.sdk".into(); },
                        KeyCode::Char('S') => {
                            if !input_str.is_empty() {
                                match fs::write(input_str.trim(), board_to_sdk(board)) { Ok(_) => status = format!("Saved {}", input_str.trim()), Err(e) => status = format!("Save failed: {}", e) }
                            }
                        },
                        KeyCode::Backspace => { if !board.cells[sel.0][sel.1].fixed { board.cells[sel.0][sel.1].value=0; } },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
}

fn board_to_sdk(b: &Board) -> String {
    let mut s = String::with_capacity(81);
    for r in 0..9 { for c in 0..9 { let v=b.cells[r][c].value; s.push(if v==0 { '.' } else { char::from(b'0'+v) }); }}
    s
}

fn super_simplify_normalize(raw: &str) -> Result<String, ()> {
    let mut out = String::with_capacity(81);
    for ch in raw.chars() {
        match ch { '1'..='9' => out.push(ch), '0'|'.' => out.push('.'), _=>{} }
        if out.len()==81 { break; }
    }
    if out.len()==81 { Ok(out) } else { Err(()) }
}
