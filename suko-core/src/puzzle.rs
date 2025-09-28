use rand::{seq::SliceRandom, Rng, SeedableRng};

use crate::board::Board;

pub struct PuzzleGenerator {
    rng: rand::rngs::StdRng,
}

impl PuzzleGenerator {
    pub fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_rng(rand::thread_rng()).unwrap(),
        };
        Self { rng }
    }

    pub fn generate_full_grid(&mut self) -> Board {
        let mut b = Board::empty();
        self.fill_grid(&mut b);
        // mark all fixed; caller may remove
        for r in 0..9 { for c in 0..9 { let v = b.cells[r][c].value; b.cells[r][c].fixed = v != 0; }}
        b
    }

    pub fn generate_puzzle(&mut self, target_clues: usize) -> Board {
        let mut b = self.generate_full_grid();
        // positions 0..80
        let mut positions: Vec<usize> = (0..81).collect();
        positions.shuffle(&mut self.rng);
        let mut clues = 81usize;
        for idx in positions {
            if clues <= target_clues { break; }
            let r = idx / 9; let c = idx % 9;
            let old = b.cells[r][c].value;
            if old == 0 { continue; }
            b.cells[r][c].value = 0;
            // Temporarily clear fixed to avoid candidate shortcuts
            b.cells[r][c].fixed = false;
            // Check uniqueness
            let mut copy = b.clone();
            let count = count_solutions(&mut copy, 2);
            if count != 1 {
                // restore
                b.cells[r][c].value = old;
            } else {
                clues -= 1;
            }
        }
        // finalize fixed flags
        for r in 0..9 { for c in 0..9 { let v = b.cells[r][c].value; b.cells[r][c].fixed = v != 0; }}
        b
    }

    fn fill_grid(&mut self, b: &mut Board) -> bool {
        if let Some((r, c)) = find_next_mrv(b) {
            let mut digits: Vec<u8> = (1..=9).collect();
            digits.shuffle(&mut self.rng);
            for d in digits {
                if b.candidates(r, c)[d as usize] {
                    b.cells[r][c].value = d;
                    if self.fill_grid(b) { return true; }
                    b.cells[r][c].value = 0;
                }
            }
            false
        } else {
            true
        }
    }
}

fn find_next_mrv(b: &Board) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize, usize)> = None; // (r,c,count)
    for r in 0..9 { for c in 0..9 { if b.cells[r][c].value == 0 {
        let cand = b.candidates(r, c);
        let mut cnt = 0; for v in 1..=9 { if cand[v as usize] { cnt += 1; } }
        if cnt == 0 { return Some((r, c)); }
        match best { None => best = Some((r,c,cnt)), Some((_,_,bc)) if cnt < bc => best = Some((r,c,cnt)), _ => {} }
    }}}
    best.map(|(r,c,_)| (r,c))
}

fn count_solutions(b: &mut Board, limit: usize) -> usize {
    fn backtrack(b: &mut Board, count: &mut usize, limit: usize) {
        if *count >= limit { return; }
        if let Some((r,c)) = find_empty(b) {
            let cand = b.candidates(r,c);
            for d in 1..=9u8 {
                if cand[d as usize] {
                    b.cells[r][c].value = d;
                    backtrack(b, count, limit);
                    b.cells[r][c].value = 0;
                    if *count >= limit { return; }
                }
            }
        } else {
            // full
            if b.is_valid() { *count += 1; }
        }
    }
    fn find_empty(b: &Board) -> Option<(usize,usize)> { for r in 0..9 { for c in 0..9 { if b.cells[r][c].value == 0 { return Some((r,c)); } }} None }
    let mut count = 0;
    backtrack(b, &mut count, limit);
    count
}
