use eframe::{egui, App, Frame, NativeOptions};
use suko_core::{board::Board, solver::{BacktrackingSolver, LogicalSolver, Solver, Step}, devlog::{SessionLog, write_session_markdown}};

struct SukoApp {
    input: String,
    board: Board,
    steps: Vec<Step>,
    step_idx: usize,
    back: BacktrackingSolver,
    logic: LogicalSolver,
    sel: (usize, usize),
    highlight_last: bool,
}

impl Default for SukoApp {
    fn default() -> Self {
        Self {
            input: String::new(),
            board: Board::empty(),
            steps: Vec::new(),
            step_idx: 0,
            back: BacktrackingSolver::new(),
            logic: LogicalSolver::new(),
            sel: (0,0),
            highlight_last: true,
        }
    }
}

impl App for SukoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.heading("Suko");
                ui.separator();
                ui.label("Input (81 chars):");
                ui.text_edit_singleline(&mut self.input);
                if ui.button(egui::RichText::new("Load").strong()).clicked() {
                    if let Ok(b) = Board::parse(&self.input) { self.board = b; self.steps.clear(); self.step_idx = 0; }
                }
                ui.separator();
                if ui.button(egui::RichText::new("Backtracking").strong()).clicked() { self.steps = self.back.solve_steps(&self.board, None); self.step_idx = 0; }
                if ui.button(egui::RichText::new("Logical step").strong()).clicked() { self.steps = self.logic.solve_steps(&self.board, Some(1)); self.step_idx=0; }
                if ui.button(egui::RichText::new("Next").strong()).clicked() { if self.step_idx < self.steps.len() { self.board = self.steps[self.step_idx].board.clone(); self.step_idx+=1; } }
                if ui.button(egui::RichText::new("Export session").strong()).clicked() {
                    if !self.steps.is_empty() {
                        let log = SessionLog { title: "Sudoku solving session".into(), puzzle: self.input.clone(), solver_name: if self.steps.iter().any(|s| matches!(s.kind, suko_core::solver::StepKind::Guess{..}| suko_core::solver::StepKind::Backtrack)) { "Backtracking".into() } else { "Logical".into() }, steps: self.steps.clone() };
                        let _ = write_session_markdown("logs/sessions", &log);
                    }
                }
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
            draw_board_ui(ui, &mut self.board, &mut self.sel, self.steps.get(self.step_idx.saturating_sub(1)), self.highlight_last);

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
    }
}

fn draw_board_ui(ui: &mut egui::Ui, board: &mut Board, sel: &mut (usize,usize), last_step: Option<&Step>, highlight_last: bool) {
    egui::Grid::new("board").num_columns(9).spacing([6.0, 4.0]).show(ui, |ui| {
        let mut last_cells: Vec<(usize,usize)> = Vec::new();
        if let Some(step) = last_step { match &step.kind { suko_core::solver::StepKind::Place{ r,c, .. } => { last_cells.push((*r,*c)); }, _ => {} } }
        for r in 0..9 {
            for c in 0..9 {
                let v = board.cells[r][c].value;
                let peers = r==sel.0 || c==sel.1 || (r/3==sel.0/3 && c/3==sel.1/3);
                let mut txt = if v==0 { "·".to_string() } else { v.to_string() };
                if *sel==(r,c) { txt = format!("[{}]", txt); }
                let mut text = egui::RichText::new(txt).size(18.0);
                if board.cells[r][c].fixed { text = text.color(egui::Color32::LIGHT_BLUE); }
                let mut button = egui::Button::new(text).min_size(egui::vec2(28.0, 28.0));
                if peers { button = button.fill(egui::Color32::from_gray(40)); }
                if highlight_last && last_cells.contains(&(r,c)) { button = button.stroke(egui::Stroke::new(2.0, egui::Color32::YELLOW)); }
                let resp = ui.add(button);
                if resp.clicked() { *sel=(r,c); }
            }
            ui.end_row();
        }
    });
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
