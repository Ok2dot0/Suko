use eframe::{egui, App, Frame, NativeOptions};
use suko_core::{board::Board, solver::BacktracingBruteSolver, puzzle::PuzzleGenerator};
use std::fs;
use std::path::PathBuf;

struct SukoApp {
    board: Board,
    sel: (usize, usize),
    puzzle_text: String,
    status: String,
    original_board: Option<Board>,
    brute: BacktracingBruteSolver,
    show_candidates: bool,
    // Puzzle generator state
    clues_target: usize,
    puzzle_seed_text: String,
}

impl Default for SukoApp {
    fn default() -> Self {
        Self {
            board: Board::empty(),
            sel: (0,0),
            puzzle_text: String::new(),
            status: String::new(),
            original_board: None,
            brute: BacktracingBruteSolver::new(),
            show_candidates: true,
            clues_target: 30,
            puzzle_seed_text: String::new(),
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
                                                self.board = b.clone(); self.sel=(0,0);
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
                if ui.button(egui::RichText::new("Backtracing Solve").strong()).on_hover_text("Bruteforce: try 9→1 on first empty cell, backtrack on conflicts").clicked() {
                    match self.brute.solve_to_completion(&self.board) {
                        Some(solved) => { self.board = solved; self.status = "Solved by backtracing".to_string(); },
                        None => { self.status = "No solution found".to_string(); }
                    }
                }
                ui.separator();
                if ui.button(egui::RichText::new("Save Board…").strong()).on_hover_text("Save current grid as 81-char .sdk").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Sudoku", &["sdk","txt"]) 
                        .set_file_name("puzzle.sdk")
                        .save_file() {
                        match fs::write(&path, board_to_sdk(&self.board)) { Ok(_) => self.status = format!("Saved board: {}", display_filename(path)), Err(e) => self.status = format!("Failed to save board: {}", e) }
                    }
                }
                ui.separator();
                if ui.button(egui::RichText::new("Clear Board").strong()).on_hover_text("Set all cells to empty").clicked() {
                    self.board = Board::empty();
                    self.sel = (0,0);
                    self.status = "Cleared board".into();
                }
                ui.separator();
                ui.checkbox(&mut self.show_candidates, "Show candidates");
                // Keep UI compact: only essential controls per user request
            });
            ui.add_space(6.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            // Sudoku toolbar (generation)
                ui.horizontal(|ui| {
                    ui.label("Clues target:");
                    ui.add(egui::Slider::new(&mut self.clues_target, 20..=40));
                    if ui.button("Generate puzzle").on_hover_text("Random puzzle with unique solution (target clues)").clicked() {
                        let mut gen = PuzzleGenerator::new(None);
                        self.board = gen.generate_puzzle(self.clues_target);
                        self.sel = (0,0);
                        self.status = format!("Generated puzzle ~{} clues", self.clues_target);
                    }
                    ui.separator();
                    ui.label("Seed:");
                    ui.text_edit_singleline(&mut self.puzzle_seed_text);
                    if ui.button("Generate seeded").clicked() {
                        if let Ok(seed) = self.puzzle_seed_text.trim().parse::<u64>() {
                            let mut gen = PuzzleGenerator::new(Some(seed));
                            self.board = gen.generate_puzzle(self.clues_target);
                            self.sel = (0,0);
                            self.status = format!("Generated seeded puzzle (seed {})", seed);
                        }
                    }
                });
                ui.separator();
                draw_board_ui(ui, &mut self.board, &mut self.sel, self.show_candidates);

            // Keyboard digit entry for selected cell
            ui.input(|i| {
                for ev in &i.events {
                    if let egui::Event::Text(t) = ev {
                        if let Some(ch) = t.chars().next() {
                            if ch == '.' || ch == '0' { if !self.board.cells[self.sel.0][self.sel.1].fixed { self.board.cells[self.sel.0][self.sel.1].value=0; } }
                            if ch.is_ascii_digit() && ('1'..='9').contains(&ch) {
                                if !self.board.cells[self.sel.0][self.sel.1].fixed {
                                    self.board.cells[self.sel.0][self.sel.1].value = ch.to_digit(10).unwrap() as u8;
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

fn draw_board_ui(ui: &mut egui::Ui, board: &mut Board, sel: &mut (usize,usize), show_candidates: bool) {
    egui::Grid::new("board").num_columns(9).spacing([4.0, 4.0]).show(ui, |ui| {
        for r in 0..9 {
            for c in 0..9 {
                let v = board.cells[r][c].value;
                let peers = r==sel.0 || c==sel.1 || (r/3==sel.0/3 && c/3==sel.1/3);
                let txt = if v==0 { "·".to_string() } else { v.to_string() };
                let mut text = egui::RichText::new(txt).size(22.0);
                if board.cells[r][c].fixed { text = text.color(egui::Color32::LIGHT_BLUE); }
                let mut button = egui::Button::new(text).min_size(egui::vec2(40.0, 40.0));
                if peers { button = button.fill(egui::Color32::from_gray(40)); }
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
