use crate::board::Board;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepKind {
    Place { r: usize, c: usize, v: u8, reason: String },
    Guess { r: usize, c: usize, v: u8 },
    Backtrack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub index: usize,
    pub kind: StepKind,
    pub board: Board,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverOutcome { Solved, Unsolvable, Incomplete }

pub trait Solver {
    fn name(&self) -> &str;
    fn solve_steps(&mut self, board: &Board, max_steps: Option<usize>) -> Vec<Step>;
}

pub struct BacktrackingSolver;
impl BacktrackingSolver {
    pub fn new() -> Self { Self }
}

impl Solver for BacktrackingSolver {
    fn name(&self) -> &str { "Backtracking" }
    fn solve_steps(&mut self, board: &Board, max_steps: Option<usize>) -> Vec<Step> {
        let mut steps = Vec::new();
        let mut b = board.clone();
        fn find_empty(b: &Board) -> Option<(usize,usize)> { for r in 0..9 { for c in 0..9 { if b.cells[r][c].value==0 { return Some((r,c)); }}} None }
        fn rec(b: &mut Board, steps: &mut Vec<Step>, idx: &mut usize, max: Option<usize>) -> bool {
            if b.is_solved() { return true; }
            if let Some(m)=max { if *idx >= m { return false; } }
            let Some((r,c)) = find_empty(b) else { return true; };
            let cand = b.candidates(r,c);
            for v in 1..=9 {
                if !cand[v as usize] { continue; }
                b.cells[r][c].value = v;
                *idx += 1;
                steps.push(Step{ index:*idx, kind: StepKind::Guess{ r, c, v }, board: b.clone() });
                if b.is_valid() && rec(b, steps, idx, max) { return true; }
                // backtrack
                b.cells[r][c].value = 0;
                *idx += 1; steps.push(Step{ index:*idx, kind: StepKind::Backtrack, board: b.clone() });
                if let Some(m)=max { if *idx >= m { return false; } }
            }
            false
        }
        let mut idx=0usize; let solved = rec(&mut b, &mut steps, &mut idx, max_steps);
        if solved { steps } else { steps }
    }
}

pub struct LogicalSolver;
impl LogicalSolver { pub fn new() -> Self { Self } }

impl Solver for LogicalSolver {
    fn name(&self) -> &str { "Logical" }
    fn solve_steps(&mut self, board: &Board, max_steps: Option<usize>) -> Vec<Step> {
        let mut b = board.clone();
        let mut steps = Vec::new();
        let mut idx=0usize;
        loop {
            if b.is_solved() { break; }
            if let Some(m)=max_steps { if idx>=m { break; } }
            let mut progress=false;
            // Strategy 1: Naked singles
            for r in 0..9 { for c in 0..9 {
                if b.cells[r][c].value==0 {
                    let cand = b.candidates(r,c);
                    let vals: Vec<u8> = (1..=9).filter(|&v| cand[v as usize]).collect();
                    if vals.len()==1 {
                        let v = vals[0];
                        b.cells[r][c].value = v; progress=true; idx+=1;
                        steps.push(Step{ index: idx, kind: StepKind::Place{ r, c, v, reason: "Naked single".into() }, board: b.clone() });
                    }
                }
            }}
            if progress { continue; }
            // Strategy 2: Hidden singles in row/col/box
            for r in 0..9 {
                let mut counts=[0u8;10]; let mut lastpos=[(0usize,0usize);10];
                for c in 0..9 { if b.cells[r][c].value==0 { let cand=b.candidates(r,c); for v in 1..=9 { if cand[v as usize] { counts[v as usize]+=1; lastpos[v as usize]=(r,c); } } } }
                for v in 1..=9 { if counts[v as usize]==1 { let (rr,cc)=lastpos[v as usize]; if b.cells[rr][cc].value==0 { b.cells[rr][cc].value=v; idx+=1; steps.push(Step{ index: idx, kind: StepKind::Place{ r:rr,c:cc,v, reason: format!("Hidden single in row {}", r+1) }, board: b.clone() }); return steps; } } }
            }
            for c in 0..9 {
                let mut counts=[0u8;10]; let mut lastpos=[(0usize,0usize);10];
                for r in 0..9 { if b.cells[r][c].value==0 { let cand=b.candidates(r,c); for v in 1..=9 { if cand[v as usize] { counts[v as usize]+=1; lastpos[v as usize]=(r,c); } } } }
                for v in 1..=9 { if counts[v as usize]==1 { let (rr,cc)=lastpos[v as usize]; if b.cells[rr][cc].value==0 { b.cells[rr][cc].value=v; idx+=1; steps.push(Step{ index: idx, kind: StepKind::Place{ r:rr,c:cc,v, reason: format!("Hidden single in col {}", c+1) }, board: b.clone() }); return steps; } } }
            }
            for br in 0..3 { for bc in 0..3 {
                let mut counts=[0u8;10]; let mut lastpos=[(0usize,0usize);10];
                for r in br*3..br*3+3 { for c in bc*3..bc*3+3 { if b.cells[r][c].value==0 { let cand=b.candidates(r,c); for v in 1..=9 { if cand[v as usize] { counts[v as usize]+=1; lastpos[v as usize]=(r,c); } } } }}
                for v in 1..=9 { if counts[v as usize]==1 { let (rr,cc)=lastpos[v as usize]; if b.cells[rr][cc].value==0 { b.cells[rr][cc].value=v; idx+=1; steps.push(Step{ index: idx, kind: StepKind::Place{ r:rr,c:cc,v, reason: format!("Hidden single in box ({},{})", br+1, bc+1) }, board: b.clone() }); return steps; } } }
            }}
            // If no progress, stop for now
            break;
        }
        steps
    }
}
