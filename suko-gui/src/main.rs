use eframe::{egui, App, Frame, NativeOptions};
use suko_core::{board::Board, solver::{BacktrackingSolver, LogicalSolver, Solver, Step}, devlog::SessionLog};
use std::fs;
use std::path::PathBuf;

struct SukoApp {
    board: Board,
    steps: Vec<Step>,
    step_idx: usize,
    back: BacktrackingSolver,
    logic: LogicalSolver,
    sel: (usize, usize),
    highlight_last: bool,
    puzzle_text: String,
    status: String,
    show_candidates: bool,
    original_board: Option<Board>,
}

impl Default for SukoApp {
    fn default() -> Self {
        Self {
            board: Board::empty(),
            steps: Vec::new(),
            step_idx: 0,
            back: BacktrackingSolver::new(),
            logic: LogicalSolver::new(),
            sel: (0,0),
            highlight_last: true,
            puzzle_text: String::new(),
            status: String::new(),
            show_candidates: false,
            original_board: None,
        }
    }
}

impl App for SukoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            // Subtle visual tweaks for a cleaner grid look
            let mut style = (*ui.ctx().style()).clone();
            style.visuals.widgets.inactive.rounding = egui::Rounding::ZERO;
            style.visuals.widgets.hovered.rounding = egui::Rounding::ZERO;
            style.visuals.widgets.active.rounding = egui::Rounding::ZERO;
            ui.set_style(style);

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.heading("Suko");
                ui.separator();
                if ui.button(egui::RichText::new("Open Puzzle…").strong()).on_hover_text("Open a .sdk or .txt with 81 characters (0/.) as blanks").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("Sudoku", &["sdk","txt"]).pick_file() {
                        match fs::read_to_string(&path) {
                            Ok(raw) => {
                                match normalize_puzzle_text(&raw) {
                                    Ok(norm) => {
                                        match Board::parse(&norm) {
                                            Ok(b) => {
                                                self.board = b.clone(); self.steps.clear(); self.step_idx=0; self.sel=(0,0);
                                                self.puzzle_text = norm;
                                                self.original_board = Some(b);
                                                self.status = format!("Loaded puzzle: {}", display_filename(path));
                                            },
                                            Err(e) => { self.status = format!("Failed to parse puzzle: {}", e); }
                                        }
                                    },
                                    Err(msg) => { self.status = format!("{}", msg); }
                                }
                            },
                            Err(e) => { self.status = format!("Failed to read file: {}", e); }
                        }
                    }
                }
                ui.separator();
                if ui.button(egui::RichText::new("Backtracking").strong()).clicked() { self.steps = self.back.solve_steps(&self.board, None); self.step_idx = 0; }
                if ui.button(egui::RichText::new("Logical step").strong()).clicked() { self.steps = self.logic.solve_steps(&self.board, Some(1)); self.step_idx=0; }
                if ui.button(egui::RichText::new("Next").strong()).clicked() { if self.step_idx < self.steps.len() { self.board = self.steps[self.step_idx].board.clone(); self.step_idx+=1; } }
                if ui.button(egui::RichText::new("Auto Solve").strong()).on_hover_text("Apply logical steps until stuck").clicked() {
                    // Run logical solver to completion (or until no progress), apply final board
                    let steps = self.logic.solve_steps(&self.board, None);
                    if !steps.is_empty() {
                        self.board = steps.last().unwrap().board.clone();
                        self.step_idx = steps.len();
                        self.steps = steps;
                    }
                }
                if ui.button(egui::RichText::new("Solve Completely").strong()).on_hover_text("Logical to finish; if stuck, backtracking").clicked() {
                    // 1) logical to completion
                    let mut steps_all = self.logic.solve_steps(&self.board, None);
                    let mut final_board = if let Some(last) = steps_all.last() { last.board.clone() } else { self.board.clone() };
                    // 2) if not solved, backtracking from the final logical board
                    if !final_board.is_solved() {
                        let bt_steps = self.back.solve_steps(&final_board, None);
                        // reindex and append
                        let mut idx = steps_all.len();
                        for s in bt_steps {
                            idx += 1;
                            steps_all.push(Step { index: idx, kind: s.kind, board: s.board });
                        }
                        if let Some(last) = steps_all.last() { final_board = last.board.clone(); }
                    }
                    self.board = final_board;
                    self.step_idx = steps_all.len();
                    self.steps = steps_all;
                }
                ui.separator();
                if ui.button(egui::RichText::new("Reset Puzzle").strong()).on_hover_text("Restore the originally loaded grid").clicked() {
                    if let Some(orig) = &self.original_board { self.board = orig.clone(); self.steps.clear(); self.step_idx=0; self.sel=(0,0); self.status = "Reset to original".to_string(); }
                }
                if ui.button(egui::RichText::new("Clear").strong()).on_hover_text("Clear all cells").clicked() {
                    self.board = Board::empty(); self.steps.clear(); self.step_idx=0; self.sel=(0,0); self.status = "Cleared board".to_string();
                }
                ui.separator();
                if ui.button(egui::RichText::new("Save Session…").strong()).on_hover_text("Save session markdown to a file").clicked() {
                    if !self.steps.is_empty() {
                        let puzzle = if self.puzzle_text.is_empty() { board_to_sdk(&self.board) } else { self.puzzle_text.clone() };
                        let log = SessionLog { title: "Sudoku solving session".into(), puzzle, solver_name: if self.steps.iter().any(|s| matches!(s.kind, suko_core::solver::StepKind::Guess{..}| suko_core::solver::StepKind::Backtrack)) { "Backtracking".into() } else { "Logical".into() }, steps: self.steps.clone() };
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Markdown", &["md"]) 
                            .set_file_name("session.md")
                            .save_file() {
                            match fs::write(&path, format_session_markdown(&log)) { Ok(_) => self.status = format!("Saved session: {}", display_filename(path)), Err(e) => self.status = format!("Failed to save session: {}", e) }
                        }
                    }
                }
                if ui.button(egui::RichText::new("Save Board…").strong()).on_hover_text("Save current grid as 81-char .sdk").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Sudoku", &["sdk","txt"]) 
                        .set_file_name("puzzle.sdk")
                        .save_file() {
                        match fs::write(&path, board_to_sdk(&self.board)) { Ok(_) => self.status = format!("Saved board: {}", display_filename(path)), Err(e) => self.status = format!("Failed to save board: {}", e) }
                    }
                }
                ui.separator();
                ui.checkbox(&mut self.show_candidates, "Show candidates");
                if let Some(prev) = self.steps.get(self.step_idx.saturating_sub(1)) {
                    match &prev.kind {
                        suko_core::solver::StepKind::Place{ reason, .. } => { ui.label(format!("Last: {}", reason)); }
                        suko_core::solver::StepKind::Guess{ .. } => { ui.label("Last: Guess"); }
                        suko_core::solver::StepKind::Backtrack => { ui.label("Last: Backtrack"); }
                    }
                }
            });
            ui.add_space(6.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.highlight_last, "Highlight last peers");
                if self.board.cells[self.sel.0][self.sel.1].value==0 {
                    let cand = self.board.candidates(self.sel.0, self.sel.1);
                    let list = (1..=9)
                        .filter(|&v| cand[v as usize])
                        .map(|v| char::from(b'0'+v))
                        .map(|ch| ch.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    ui.label(format!("Sel ({}, {}) cands: {}", self.sel.0+1, self.sel.1+1, list));
                } else {
                    ui.label(format!("Sel ({}, {}) value: {}", self.sel.0+1, self.sel.1+1, self.board.cells[self.sel.0][self.sel.1].value));
                }
            });

            ui.add_space(8.0);
            draw_board_ui(ui, &mut self.board, &mut self.sel, self.steps.get(self.step_idx.saturating_sub(1)), self.highlight_last, self.show_candidates);

            // Keyboard digit entry for selected cell
            ui.input(|i| {
                for ev in &i.events {
                    if let egui::Event::Text(t) = ev {
                        if let Some(ch) = t.chars().next() {
                            if ch == '.' || ch == '0' { if !self.board.cells[self.sel.0][self.sel.1].fixed { self.board.cells[self.sel.0][self.sel.1].value=0; self.steps.clear(); self.step_idx=0; } }
                            if ch.is_ascii_digit() && ('1'..='9').contains(&ch) {
                                if !self.board.cells[self.sel.0][self.sel.1].fixed {
                                    self.board.cells[self.sel.0][self.sel.1].value = ch.to_digit(10).unwrap() as u8;
                                    self.steps.clear(); self.step_idx=0;
                                }
                            }
                        }
                    }
                }
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let msg = if self.status.is_empty() { "Ready".to_string() } else { self.status.clone() };
                ui.label(egui::RichText::new(msg).italics());
            });
            ui.add_space(4.0);
        });
    }
}

fn draw_board_ui(ui: &mut egui::Ui, board: &mut Board, sel: &mut (usize,usize), last_step: Option<&Step>, highlight_last: bool, show_candidates: bool) {
    egui::Grid::new("board").num_columns(9).spacing([4.0, 4.0]).show(ui, |ui| {
        let mut last_cells: Vec<(usize,usize)> = Vec::new();
        if let Some(step) = last_step { match &step.kind { suko_core::solver::StepKind::Place{ r,c, .. } => { last_cells.push((*r,*c)); }, _ => {} } }
        for r in 0..9 {
            for c in 0..9 {
                let v = board.cells[r][c].value;
                let peers = r==sel.0 || c==sel.1 || (r/3==sel.0/3 && c/3==sel.1/3);
                let txt = if v==0 { "·".to_string() } else { v.to_string() };
                let mut text = egui::RichText::new(txt).size(22.0);
                if board.cells[r][c].fixed { text = text.color(egui::Color32::LIGHT_BLUE); }
                let mut button = egui::Button::new(text).min_size(egui::vec2(40.0, 40.0));
                if peers { button = button.fill(egui::Color32::from_gray(40)); }
                if highlight_last && last_cells.contains(&(r,c)) { button = button.stroke(egui::Stroke::new(2.0, egui::Color32::YELLOW)); }
                if *sel==(r,c) {
                    button = button.fill(egui::Color32::from_gray(60)).stroke(egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE));
                }
                let resp = ui.add(button);
                if resp.clicked() { *sel=(r,c); }

                // Draw grid lines around the cell
                let stroke_thin = egui::Stroke::new(1.0, egui::Color32::from_gray(90));
                let stroke_thick = egui::Stroke::new(2.0, egui::Color32::LIGHT_GRAY);
                let rect = resp.rect;
                let p = ui.painter();
                // Left border (thick at c==0 or c%3==0)
                if c == 0 || c % 3 == 0 { p.line_segment([rect.left_top(), rect.left_bottom()], stroke_thick); }
                else { p.line_segment([rect.left_top(), rect.left_bottom()], stroke_thin); }
                // Top border (thick at r==0 or r%3==0)
                if r == 0 || r % 3 == 0 { p.line_segment([rect.left_top(), rect.right_top()], stroke_thick); }
                else { p.line_segment([rect.left_top(), rect.right_top()], stroke_thin); }
                // Right border (thick at c==8 or c%3==2)
                if c == 8 || c % 3 == 2 { p.line_segment([rect.right_top(), rect.right_bottom()], stroke_thick); }
                else { p.line_segment([rect.right_top(), rect.right_bottom()], stroke_thin); }
                // Bottom border (thick at r==8 or r%3==2)
                if r == 8 || r % 3 == 2 { p.line_segment([rect.left_bottom(), rect.right_bottom()], stroke_thick); }
                else { p.line_segment([rect.left_bottom(), rect.right_bottom()], stroke_thin); }

                // Candidates (pencil marks)
                if show_candidates && board.cells[r][c].value == 0 {
                    let cand = board.candidates(r,c);
                    // Draw 3x3 tiny digits
                    let w = rect.width(); let h = rect.height();
                    for v in 1..=9 {
                        if cand[v as usize] {
                            let rr = (v-1) / 3; let cc = (v-1) % 3;
                            let x = rect.left() + (cc as f32 + 0.5) * (w/3.0);
                            let y = rect.top() + (rr as f32 + 0.55) * (h/3.0);
                            let pos = egui::pos2(x, y);
                            let font = egui::FontId::monospace(11.0);
                            p.text(pos, egui::Align2::CENTER_CENTER, format!("{}", v), font, egui::Color32::from_gray(170));
                        }
                    }
                }
            }
            ui.end_row();
        }
    });
}

fn board_to_sdk(b: &Board) -> String {
    let mut s = String::with_capacity(81);
    for r in 0..9 { for c in 0..9 { let v=b.cells[r][c].value; s.push(if v==0 { '.' } else { char::from(b'0'+v) }); }}
    s
}

fn format_session_markdown(log: &SessionLog) -> String {
    // Reuse the same format as write_session_markdown, but return as String for direct save
    let mut out = String::new();
    out.push_str(&format!("# {}\n", log.title));
    out.push_str(&format!("Solver: {}\n", log.solver_name));
    out.push_str(&format!("Puzzle: `{}`\n\n", log.puzzle));
    out.push_str("## Steps\n");
    for s in &log.steps {
        out.push_str(&format!("\n### Step {}\n", s.index));
        match &s.kind {
            suko_core::solver::StepKind::Place{ r,c,v,reason } => out.push_str(&format!("- Place {} at ({}, {}) — {}\n", v, r+1, c+1, reason)),
            suko_core::solver::StepKind::Guess{ r,c,v } => out.push_str(&format!("- Guess {} at ({}, {})\n", v, r+1, c+1)),
            suko_core::solver::StepKind::Backtrack => out.push_str("- Backtrack\n"),
        }
        out.push_str(&format!("\n``\n{}\n``\n", s.board));
    }
    out
}

fn normalize_puzzle_text(raw: &str) -> Result<String, String> {
    let mut out = String::with_capacity(81);
    for ch in raw.chars() {
        match ch {
            '1'..='9' => out.push(ch),
            '0' | '.' => out.push('.'),
            _ => {}
        }
        if out.len() == 81 { break; }
    }
    if out.len() != 81 {
        return Err(format!("Puzzle must contain 81 characters (digits or .): got {}", out.len()));
    }
    Ok(out)
}

fn display_filename(path: PathBuf) -> String {
    path.file_name().and_then(|s| s.to_str()).unwrap_or("file").to_string()
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    let options = NativeOptions::default();
    eframe::run_native(
        "Suko GUI",
        options,
        Box::new(|_| -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Box::new(SukoApp::default()))
        }),
    )
}
