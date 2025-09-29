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

/// A simple brute-force backtracer that follows the exact behavior requested:
/// - Scan cells row-major; for the first empty cell, try values 9 down to 1
/// - If a value doesn't cause conflicts, move to the next empty cell and repeat
/// - If a value conflicts, decrease to the next lower value
/// - If all values 9..1 conflict for the current cell, backtrack to the previous empty cell and decrease it
/// - Continue until all cells are filled or no solution exists
pub struct BacktracingBruteSolver;
impl BacktracingBruteSolver {
    pub fn new() -> Self { Self }

    /// Solve to completion; returns Some(solved_board) if solved, else None
    pub fn solve_to_completion(&mut self, board: &Board) -> Option<Board> {
        let mut b = board.clone();
        // Collect empty cell positions in row-major order
        let mut empties: Vec<(usize, usize)> = Vec::new();
        for r in 0..9 { for c in 0..9 { if b.cells[r][c].value == 0 { empties.push((r,c)); } } }
        if empties.is_empty() { return Some(b); }

        // Track the next value to try for each empty cell. Start above 9 so we try 9 first.
        let mut next: Vec<u8> = vec![10u8; empties.len()];
        let mut i: isize = 0; // index into empties

        while i >= 0 && (i as usize) < empties.len() {
            let idx = i as usize;
            let (r, c) = empties[idx];

            let mut placed = false;
            while next[idx] > 1 {
                next[idx] -= 1; // try next lower value (starts from 9)
                let v = next[idx];
                b.cells[r][c].value = v;
                if b.is_valid() {
                    // accept and move forward
                    placed = true;
                    i += 1;
                    break;
                }
            }

            if !placed {
                // reset this cell and backtrack
                b.cells[r][c].value = 0;
                next[idx] = 10; // reset for future revisits
                i -= 1;
                // When we step back, we will decrease the previous cell further on the next iteration
            }
        }

        if i < 0 { return None; }
        // i == empties.len() => all placed
        if b.is_valid() { Some(b) } else { None }
    }
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
        // Minimum Remaining Values (MRV): pick the empty cell with the fewest candidates (>0). If any empty cell has 0 candidates, fail fast.
        fn find_mrv(b: &Board) -> Option<(usize,usize,[bool;10], usize)> {
            let mut best: Option<(usize,usize,[bool;10], usize)> = None;
            for r in 0..9 { for c in 0..9 {
                if b.cells[r][c].value==0 {
                    let cand = b.candidates(r,c);
                    let count = (1..=9).filter(|&v| cand[v as usize]).count();
                    if count==0 { return Some((r,c,cand,0)); }
                    match best {
                        None => best = Some((r,c,cand,count)),
                        Some((_,_,_,bc)) if count < bc => best = Some((r,c,cand,count)),
                        _ => {}
                    }
                }
            }}
            best
        }
        fn any_zero_candidate(b: &Board) -> bool {
            for r in 0..9 { for c in 0..9 { if b.cells[r][c].value==0 {
                let cand=b.candidates(r,c);
                if (1..=9).all(|v| !cand[v as usize]) { return true; }
            }}}
            false
        }
        fn rec(b: &mut Board, steps: &mut Vec<Step>, idx: &mut usize, max: Option<usize>) -> bool {
            if b.is_solved() { return true; }
            if let Some(m)=max { if *idx >= m { return false; } }
            let Some((r,c,cand,_cnt)) = find_mrv(b) else { return true; };
            if (1..=9).all(|v| !cand[v as usize]) { return false; }
            for v in 1..=9 {
                if !cand[v as usize] { continue; }
                b.cells[r][c].value = v;
                *idx += 1;
                steps.push(Step{ index:*idx, kind: StepKind::Guess{ r, c, v }, board: b.clone() });
                if b.is_valid() && !any_zero_candidate(b) && rec(b, steps, idx, max) { return true; }
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
    // Try to derive a single for any cell by applying human-style reductions
    for r in 0..9 { for c in 0..9 { if b.cells[r][c].value==0 {
        let mut mask = mask_from_candidates(b.candidates(r,c));
        if mask.count_ones() <= 1 { continue; }
        // Iterate reductions until stable (at most 9 bits)
        loop {
            let before = mask;
            mask = apply_locked_pointing_claiming(b, r, c, mask);
            mask = apply_naked_pairs_all_units(b, r, c, mask);
            if mask == before { break; }
            if mask.count_ones() == 1 { break; }
        }
        if mask.count_ones()==1 {
            let v = (1..=9).find(|&v| (mask & (1<<(v as u16)))!=0 ).unwrap();
            return Some((r,c,v as u8, "Single after reductions (pointing/claiming, pairs)".into()));
        }
    }}}
    None
}

fn mask_from_candidates(cand: [bool;10]) -> u16 { let mut m=0u16; for v in 1..=9 { if cand[v as usize] { m |= 1u16<<v; } } m }

fn apply_locked_pointing_claiming(b: &Board, r: usize, c: usize, mut mask: u16) -> u16 {
    // Implement pointing (box -> row/col) and claiming (row/col -> box) to eliminate candidates for (r,c)
    let br = r/3; let bc = c/3;
    for v in 1..=9 {
        let bit = 1u16<<v;
        if mask & bit == 0 { continue; }

        // Pointing: within box (br,bc), if all candidates for v lie in a single row r0 (relative 0..2),
        // then eliminate v from other boxes in that same row. If (r,c) is in that row but not this box, remove v.
        {
            let mut rows_in_box = [false;3];
            let mut cols_in_box = [false;3];
            for rr in br*3..br*3+3 { for cc in bc*3..bc*3+3 {
                if b.cells[rr][cc].value==0 && b.candidates(rr,cc)[v as usize] {
                    rows_in_box[rr%3]=true; cols_in_box[cc%3]=true;
                }
            }}
            let r_count = rows_in_box.iter().filter(|x| **x).count();
            let c_count = cols_in_box.iter().filter(|x| **x).count();
            if r_count==1 {
                let r_rel = (0..3).find(|i| rows_in_box[*i]).unwrap();
                let r0 = br*3 + r_rel;
                if r==r0 && c/3 != bc { mask &= !bit; }
            }
            if c_count==1 {
                let c_rel = (0..3).find(|i| cols_in_box[*i]).unwrap();
                let c0 = bc*3 + c_rel;
                if c==c0 && r/3 != br { mask &= !bit; }
            }
        }

        // Claiming from row: if in row r0, all candidates for v lie in a single box (br0, bc0), then
        // cells in that box but not in row r0 cannot be v. If (r,c) is in that box and r!=r0, remove v.
        for r0 in 0..9 {
            let mut boxes = [false;3];
            for cc in 0..9 {
                if b.cells[r0][cc].value==0 && b.candidates(r0,cc)[v as usize] { boxes[cc/3]=true; }
            }
            if boxes.iter().filter(|x| **x).count()==1 {
                let bc0 = (0..3).find(|i| boxes[*i]).unwrap();
                let br0 = r0/3;
                if r/3==br0 && c/3==bc0 && r!=r0 { mask &= !bit; }
            }
        }
        // Claiming from col: similar logic
        for c0 in 0..9 {
            let mut boxes = [false;3];
            for rr in 0..9 {
                if b.cells[rr][c0].value==0 && b.candidates(rr,c0)[v as usize] { boxes[rr/3]=true; }
            }
            if boxes.iter().filter(|x| **x).count()==1 {
                let br0 = (0..3).find(|i| boxes[*i]).unwrap();
                let bc0 = c0/3;
                if r/3==br0 && c/3==bc0 && c!=c0 { mask &= !bit; }
            }
        }
    }
    mask
}

fn apply_naked_pairs_all_units(b: &Board, r: usize, c: usize, mask: u16) -> u16 {
    let br=r/3; let bc=c/3;
    let mut m = mask;
    m = reduce_by_naked_pairs_unit(m, unit_candidates_row(b, r));
    m = reduce_by_naked_pairs_unit(m, unit_candidates_col(b, c));
    m = reduce_by_naked_pairs_unit(m, unit_candidates_box(b, br, bc));
    m
}
fn unit_candidates_row(b: &Board, r: usize) -> [[bool;10];9] { let mut out=[[false;10];9]; for c in 0..9 { out[c]=b.candidates(r,c); } out }
fn unit_candidates_col(b: &Board, c: usize) -> [[bool;10];9] { let mut out=[[false;10];9]; for r in 0..9 { out[r]=b.candidates(r,c); } out }
fn unit_candidates_box(b: &Board, br: usize, bc: usize) -> [[bool;10];9] { let mut out=[[false;10];9]; let mut i=0; for r in br*3..br*3+3 { for c in bc*3..bc*3+3 { out[i]=b.candidates(r,c); i+=1; }} out }

fn reduce_by_naked_pairs_unit(current_mask: u16, unit_cands: [[bool;10];9]) -> u16 {
    // Find any pair of cells in unit that have exactly two same candidates {a,b}. Eliminate a,b from other cells (including possibly (r,c)).
    let mut pairs = Vec::new();
    for i in 0..9 {
        let m = mask_from_candidates(unit_cands[i]);
        if m.count_ones()==2 { pairs.push(m); }
    }
    // For each candidate pair that occurs in exactly two cells, it's a valid naked pair
    let valids: Vec<u16> = pairs.into_iter().filter(|&pm| unit_cands.iter().filter(|cand| mask_from_candidates(**cand)==pm).count()==2).collect();
    if valids.is_empty() { return current_mask; }
    // If current cell has one of the pair candidates and is not part of the pair, eliminate them.
    let mut new_mask = current_mask;
    for &pm in &valids {
        if current_mask != pm { new_mask &= !pm; }
    }
    new_mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn reduces_by_naked_pairs_in_row_to_single() {
        // Construct a unit with a naked pair {1,2} in two cells and a target cell {1,2,3}
        // Expect: pair eliminates 1 and 2 from the target, leaving single {3}.
        let mut unit = [[false; 10]; 9];
        // Pair cells
        unit[0][1] = true; unit[0][2] = true; // {1,2}
        unit[1][1] = true; unit[1][2] = true; // {1,2}
        // Target cell with {1,2,3}
        unit[2][1] = true; unit[2][2] = true; unit[2][3] = true;
        // Fill remaining with some other non-pair candidate to avoid false pairs
        for i in 3..9 { unit[i][4] = true; } // arbitrary {4}

        let current_mask: u16 = (1u16 << 1) | (1u16 << 2) | (1u16 << 3);
        let new_mask = reduce_by_naked_pairs_unit(current_mask, unit);
        assert_eq!(new_mask.count_ones(), 1, "mask should reduce to a single");
        assert!(new_mask & (1u16 << 3) != 0, "remaining candidate should be 3");
    }

    #[test]
    fn claiming_row_eliminates_candidate_in_box() {
        // Set up a board where in row 0, candidate '5' appears only in box (0,0),
        // so 'claiming' should eliminate 5 from other cells in that box outside row 0.
        let mut b = Board::empty();
        // Place 5s in columns 3..8 (boxes 1 and 2) on a different row to block '5'
        // in row 0 outside box (0,0), without affecting row 1 where our target cell sits.
        for col in 3..=8 { b.cells[2][col].value = 5; }

        // Target cell inside the claimed box but not in row 0
        let r = 1usize; let c = 0usize; let v = 5u8;
        assert!(b.candidates(r, c)[v as usize], "precondition: v should be a candidate initially");
        let mut mask = 0u16;
        for vv in 1..=9 { if b.candidates(r, c)[vv as usize] { mask |= 1u16 << vv; } }

        let new_mask = apply_locked_pointing_claiming(&b, r, c, mask);
        assert!(new_mask & (1u16 << v) == 0, "candidate {} should be eliminated by claiming", v);
    }
}
