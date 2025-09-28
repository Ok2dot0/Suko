use eframe::{egui, App, Frame, NativeOptions};
use suko_core::{board::Board, solver::{BacktrackingSolver, LogicalSolver, Solver, Step}, devlog::{SessionLog, write_session_markdown}};

struct SukoApp {
    input: String,
    board: Board,
    steps: Vec<Step>,
    step_idx: usize,
    back: BacktrackingSolver,
    logic: LogicalSolver,
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
        }
    }
}

impl App for SukoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Suko GUI");
            ui.horizontal(|ui| {
                ui.label("Input (81 chars):");
                ui.text_edit_singleline(&mut self.input);
                if ui.button("Load").clicked() {
                    if let Ok(b) = Board::parse(&self.input) { self.board = b; self.steps.clear(); self.step_idx = 0; }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Backtracking").clicked() { self.steps = self.back.solve_steps(&self.board, None); self.step_idx = 0; }
                if ui.button("Logical step").clicked() { self.steps = self.logic.solve_steps(&self.board, Some(1)); self.step_idx=0; }
                if ui.button("Next").clicked() { if self.step_idx < self.steps.len() { self.board = self.steps[self.step_idx].board.clone(); self.step_idx+=1; } }
                if ui.button("Export session").clicked() {
                    if !self.steps.is_empty() {
                        let log = SessionLog { title: "Sudoku solving session".into(), puzzle: self.input.clone(), solver_name: if self.steps.iter().any(|s| matches!(s.kind, suko_core::solver::StepKind::Guess{..}| suko_core::solver::StepKind::Backtrack)) { "Backtracking".into() } else { "Logical".into() }, steps: self.steps.clone() };
                        let _ = write_session_markdown("logs/sessions", &log);
                    }
                }
            });

            draw_board_ui(ui, &self.board);
        });
    }
}

fn draw_board_ui(ui: &mut egui::Ui, board: &Board) {
    egui::Grid::new("board").num_columns(9).show(ui, |ui| {
        for r in 0..9 {
            for c in 0..9 {
                let v = board.cells[r][c].value;
                ui.label(if v==0 { "·".to_string() } else { v.to_string() });
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
