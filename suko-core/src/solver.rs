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
        // produce at most one logical step unless max_steps allows more
        while !b.is_solved() {
            if let Some(m)=max_steps { if idx>=m { break; } }
            // Strategy priority:
            // 1) Naked singles
            if let Some((r,c,v,reason)) = find_naked_single(&b) {
                apply_place(&mut b, r, c, v);
                idx+=1; steps.push(Step{ index: idx, kind: StepKind::Place{ r,c,v,reason }, board: b.clone() });
                continue;
            }
            // 2) Hidden singles
            if let Some((r,c,v,reason)) = find_hidden_single(&b) {
                apply_place(&mut b, r, c, v);
                idx+=1; steps.push(Step{ index: idx, kind: StepKind::Place{ r,c,v,reason }, board: b.clone() });
                continue;
            }
            // 3) Reductions (locked candidates pointing/claiming, naked pairs) leading to a single
            if let Some((r,c,v,reason)) = find_single_after_reductions(&b) {
                apply_place(&mut b, r, c, v);
                idx+=1; steps.push(Step{ index: idx, kind: StepKind::Place{ r,c,v,reason }, board: b.clone() });
                continue;
            }
            break;
        }
        steps
    }
}

fn apply_place(b: &mut Board, r: usize, c: usize, v: u8) { b.cells[r][c].value = v; }

fn find_naked_single(b: &Board) -> Option<(usize,usize,u8,String)> {
    for r in 0..9 { for c in 0..9 { if b.cells[r][c].value==0 {
        let cand = b.candidates(r,c);
        let vals: Vec<u8> = (1..=9).filter(|&v| cand[v as usize]).collect();
        if vals.len()==1 { return Some((r,c,vals[0], "Naked single".into())); }
    }}}
    None
}

fn find_hidden_single(b: &Board) -> Option<(usize,usize,u8,String)> {
    // row
    for r in 0..9 {
        let mut counts=[0u8;10]; let mut lastpos=[(0usize,0usize);10];
        for c in 0..9 { if b.cells[r][c].value==0 { let cand=b.candidates(r,c); for v in 1..=9 { if cand[v as usize] { counts[v as usize]+=1; lastpos[v as usize]=(r,c); } } } }
        for v in 1..=9 { if counts[v as usize]==1 { let (rr,cc)=lastpos[v as usize]; if b.cells[rr][cc].value==0 { return Some((rr,cc,v, format!("Hidden single in row {}", r+1))); } } }
    }
    // col
    for c in 0..9 {
        let mut counts=[0u8;10]; let mut lastpos=[(0usize,0usize);10];
        for r in 0..9 { if b.cells[r][c].value==0 { let cand=b.candidates(r,c); for v in 1..=9 { if cand[v as usize] { counts[v as usize]+=1; lastpos[v as usize]=(r,c); } } } }
        for v in 1..=9 { if counts[v as usize]==1 { let (rr,cc)=lastpos[v as usize]; if b.cells[rr][cc].value==0 { return Some((rr,cc,v, format!("Hidden single in col {}", c+1))); } } }
    }
    // box
    for br in 0..3 { for bc in 0..3 {
        let mut counts=[0u8;10]; let mut lastpos=[(0usize,0usize);10];
        for r in br*3..br*3+3 { for c in bc*3..bc*3+3 { if b.cells[r][c].value==0 { let cand=b.candidates(r,c); for v in 1..=9 { if cand[v as usize] { counts[v as usize]+=1; lastpos[v as usize]=(r,c); } } } }}
        for v in 1..=9 { if counts[v as usize]==1 { let (rr,cc)=lastpos[v as usize]; if b.cells[rr][cc].value==0 { return Some((rr,cc,v, format!("Hidden single in box ({},{})", br+1, bc+1))); } } }
    }}
    None
}

fn find_single_after_reductions(b: &Board) -> Option<(usize,usize,u8,String)> {
    for r in 0..9 { for c in 0..9 { if b.cells[r][c].value==0 {
        let base = mask_from_candidates(b.candidates(r,c));
        if base.count_ones() <= 1 { continue; }
        let reduced = apply_locked_and_pairs_reductions(b, r, c, base);
        if reduced.count_ones()==1 {
            let v = (1..=9).find(|&v| (reduced & (1<<(v as u16)))!=0 ).unwrap();
            return Some((r,c,v as u8, "Single after reductions (locked/pairs)".into()));
        }
    }}}
    None
}

fn mask_from_candidates(cand: [bool;10]) -> u16 { let mut m=0u16; for v in 1..=9 { if cand[v as usize] { m |= 1u16<<v; } } m }

fn apply_locked_and_pairs_reductions(b: &Board, r: usize, c: usize, mut mask: u16) -> u16 {
    // Locked candidates (pointing/claiming) and naked pairs in units containing (r,c)
    // Pointing/claiming in box affecting row/col
    let br=r/3; let bc=c/3;
    for v in 1..=9 {
        let bit = 1u16<<v;
        if mask & bit == 0 { continue; }
        // Collect candidate positions for v in this box
        let mut rows_in_box = [false;3];
        let mut cols_in_box = [false;3];
        for rr in br*3..br*3+3 { for cc in bc*3..bc*3+3 {
            if b.cells[rr][cc].value==0 && b.candidates(rr,cc)[v as usize] { rows_in_box[rr%3]=true; cols_in_box[cc%3]=true; }
        }}
        let rows_count = rows_in_box.iter().filter(|x| **x).count();
        let cols_count = cols_in_box.iter().filter(|x| **x).count();
        // If v is confined to a single row within the box, then v cannot appear elsewhere in that row in other boxes.
        if rows_count==1 {
            let confined_row = (0..3).find(|i| rows_in_box[*i]).unwrap();
            if r%3 != confined_row { /* not our row */ } else {
                // If our cell is outside this box but same row, we would eliminate; here our cell is inside the box, so no elimination.
                // However, for cells in same row but other boxes, eliminate v. If (r,c) is one of those, eliminate.
                // Our (r,c) is in same row; if c/3 != bc then eliminate.
                if c/3 != bc { mask &= !bit; }
            }
        }
        // If v is confined to a single column within the box
        if cols_count==1 {
            let confined_col = (0..3).find(|i| cols_in_box[*i]).unwrap();
            if c%3 != confined_col { /* not our col */ } else {
                if r/3 != br { mask &= !bit; }
            }
        }
    }
    // Naked pairs in row, column, and box
    // Row
    mask = reduce_by_naked_pairs_unit(mask, vmask_row(b, r), unit_candidates_row(b, r), (r,c), Unit::Row);
    // Col
    mask = reduce_by_naked_pairs_unit(mask, vmask_col(b, c), unit_candidates_col(b, c), (r,c), Unit::Col);
    // Box
    mask = reduce_by_naked_pairs_unit(mask, vmask_box(b, br, bc), unit_candidates_box(b, br, bc), (r,c), Unit::Box);
    mask
}

#[derive(Copy,Clone)]
enum Unit { Row, Col, Box }

fn vmask_row(b: &Board, r: usize) -> u16 { let mut m=0u16; for c in 0..9 { let cc=b.candidates(r,c); for v in 1..=9 { if b.cells[r][c].value==0 && cc[v as usize] { m |= 1u16<<v; }}} m }
fn vmask_col(b: &Board, c: usize) -> u16 { let mut m=0u16; for r in 0..9 { let cc=b.candidates(r,c); for v in 1..=9 { if b.cells[r][c].value==0 && cc[v as usize] { m |= 1u16<<v; }}} m }
fn vmask_box(b: &Board, br: usize, bc: usize) -> u16 { let mut m=0u16; for r in br*3..br*3+3 { for c in bc*3..bc*3+3 { let cc=b.candidates(r,c); for v in 1..=9 { if b.cells[r][c].value==0 && cc[v as usize] { m |= 1u16<<v; }}}} m }

fn unit_candidates_row(b: &Board, r: usize) -> [[bool;10];9] { let mut out=[[false;10];9]; for c in 0..9 { out[c]=b.candidates(r,c); } out }
fn unit_candidates_col(b: &Board, c: usize) -> [[bool;10];9] { let mut out=[[false;10];9]; for r in 0..9 { out[r]=b.candidates(r,c); } out }
fn unit_candidates_box(b: &Board, br: usize, bc: usize) -> [[bool;10];9] { let mut out=[[false;10];9]; let mut i=0; for r in br*3..br*3+3 { for c in bc*3..bc*3+3 { out[i]=b.candidates(r,c); i+=1; }} out }

fn reduce_by_naked_pairs_unit(mask: u16, _vmask: u16, unit_cands: [[bool;10];9], _pos: (usize,usize), _unit: Unit) -> u16 {
    // Find any pair of cells in unit that have exactly two same candidates {a,b}. Eliminate a,b from other cells (including possibly (r,c)).
    let mut pair_mask = 0u16;
    let mut pairs = Vec::new();
    for i in 0..9 {
        let m = mask_from_candidates(unit_cands[i]);
        if m.count_ones()==2 { pairs.push(m); }
    }
    // For each candidate pair that occurs in exactly two cells, it's a valid naked pair
    for &pm in &pairs {
        let occur = unit_cands.iter().filter(|cand| mask_from_candidates(**cand)==pm).count();
        if occur==2 { pair_mask |= pm; }
    }
    if pair_mask==0 { return mask; }
    // If current cell has one of the pair candidates and is not part of the pair, eliminate them.
    // We don't know if current cell is part of the exact pair; conservative: if its candidate set equals pm, keep; else remove pair bits.
    let current_mask = mask;
    let mut new_mask = current_mask;
    for &pm in &pairs {
        let occur = unit_cands.iter().filter(|cand| mask_from_candidates(**cand)==pm).count();
        if occur==2 {
            if current_mask != pm { new_mask &= !pm; }
        }
    }
    new_mask
}
