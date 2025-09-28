use std::io;
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{prelude::*, widgets::*};
use suko_core::{board::Board, solver::{BacktrackingSolver, LogicalSolver, Solver, Step}, devlog::{SessionLog, write_session_markdown}};

fn draw_board(frame: &mut Frame, area: Rect, board: &Board) {
    let mut lines = Vec::new();
    for r in 0..9 {
        let mut row = String::new();
        for c in 0..9 {
            let v = board.cells[r][c].value;
            row.push(if v==0 { '.' } else { char::from(b'0'+v) });
            row.push(' ');
            if c%3==2 { row.push('|'); row.push(' '); }
        }
        lines.push(Line::from(row));
        if r%3==2 { lines.push(Line::from("")); }
    }
    let block = Block::default().borders(Borders::ALL).title("Sudoku");
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input_str = String::new();
    let mut back = BacktrackingSolver::new();
    let mut logic = LogicalSolver::new();
    let mut board = Board::empty();
    let mut steps: Vec<Step> = Vec::new();
    let mut step_idx: usize = 0;

    let res = run_app(&mut terminal, &mut board, &mut input_str, &mut back, &mut logic, &mut steps, &mut step_idx);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res { eprintln!("Error: {err:#}"); }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, board: &mut Board, input_str: &mut String, back: &mut BacktrackingSolver, logic: &mut LogicalSolver, steps: &mut Vec<Step>, step_idx: &mut usize) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(18),
                    Constraint::Length(5),
                    Constraint::Min(3),
                ]).split(f.size());
            draw_board(f, chunks[0], board);
            let last_reason = steps.get(step_idx.saturating_sub(1)).map(|s| match &s.kind { suko_core::solver::StepKind::Place{ reason, .. } => reason.as_str(), suko_core::solver::StepKind::Guess{..} => "Guess", suko_core::solver::StepKind::Backtrack => "Backtrack"}).unwrap_or("");
            let help_text = format!("Commands: i=load input; b=backtracking solve; l=logical step; n=next step; s=save session; q=quit\nLast step: {}", last_reason);
            let help = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(help, chunks[1]);
            let input = Paragraph::new(input_str.as_str()).block(Block::default().borders(Borders::ALL).title("Input (81 chars, . or 0 for empty)"));
            f.render_widget(input, chunks[2]);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(k) => match k.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('i') => { if let Ok(b) = Board::parse(&input_str) { *board = b; *steps = Vec::new(); *step_idx=0; } },
                    KeyCode::Char('b') => { *steps = back.solve_steps(board, None); *step_idx=0; },
                    KeyCode::Char('l') => { let s = logic.solve_steps(board, Some(1)); if !s.is_empty() { *steps = s; *step_idx=0; } },
                    KeyCode::Char('n') => {
                        if *step_idx < steps.len() { let s = &steps[*step_idx]; *board = s.board.clone(); *step_idx += 1; }
                    },
                    KeyCode::Char('s') => {
                        if !steps.is_empty() {
                            let title = "Sudoku solving session".to_string();
                            let log = SessionLog { title, puzzle: input_str.clone(), solver_name: if steps.iter().any(|s| matches!(s.kind, suko_core::solver::StepKind::Guess{..}| suko_core::solver::StepKind::Backtrack)) { "Backtracking".into() } else { "Logical".into() }, steps: steps.clone() };
                            let _ = write_session_markdown("logs/sessions", &log);
                        }
                    },
                    KeyCode::Backspace => { input_str.pop(); },
                    KeyCode::Char(ch) => { if input_str.len()<200 { input_str.push(ch); } },
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
