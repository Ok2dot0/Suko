use crate::{grid::{Grid, Pos, bitcount, first_bit}, logger::DevLogger};
use anyhow::Result;

#[derive(Clone, Copy, Debug)]
pub enum SolveMode { LogicalOnly, SearchOnly, Hybrid }

#[derive(Clone, Copy, Debug)]
pub enum Algorithm { Logical, Backtracking }

pub struct Solver { mode: SolveMode }

impl Solver {
    pub fn new(mode: SolveMode) -> Self { Self { mode } }

    pub fn solve(&mut self, grid: &mut Grid, logger: &mut DevLogger) -> Result<()> {
        // Always (re)infer candidates for a clean start
        grid.infer_all_candidates()?;
        logger.log("Initialization", &format!("Starting grid:\n{}", grid.to_pretty_string()))?;

        match self.mode {
            SolveMode::LogicalOnly => self.solve_logically(grid, logger),
            SolveMode::SearchOnly => self.solve_search(grid, logger),
            SolveMode::Hybrid => {
                self.solve_logically(grid, logger)?;
                if !grid.is_solved() { self.solve_search(grid, logger)?; }
                Ok(())
            }
        }
    }

    pub fn solve_logically(&mut self, grid: &mut Grid, logger: &mut DevLogger) -> Result<()> {
        use crate::utils::strategies::*;
        loop {
            let before = grid.to_compact();
            // Apply a sequence of strategies
            if naked_single(grid, logger)? { continue; }
            if hidden_single(grid, logger)? { continue; }
            if naked_pair(grid, logger)? { continue; }
            if pointing_pair_triple(grid, logger)? { continue; }
            if box_line_reduction(grid, logger)? { continue; }
            if x_wing(grid, logger)? { continue; }
            let after = grid.to_compact();
            if before == after { break; }
        }
        Ok(())
    }

    pub fn solve_search(&mut self, grid: &mut Grid, logger: &mut DevLogger) -> Result<()> {
        // backtracking with MRV and forward checking
        fn search(grid: &mut Grid, logger: &mut DevLogger, depth: usize) -> Result<bool> {
            if grid.is_solved() { return Ok(true); }
            // find MRV cell
            let mut best_idx: Option<usize> = None;
            let mut best_count = 10u32;
            for i in 0..81 {
                if grid.cells[i]==0 {
                    let bc = bitcount(grid.cands[i]);
                    if bc == 0 { return Ok(false); }
                    if bc < best_count { best_count = bc; best_idx = Some(i); if bc==1 { break; } }
                }
            }
            let i = match best_idx { Some(i)=>i, None=>return Ok(true) };
            let candmask = grid.cands[i];
            let mut tried = vec![];
            for d in 1..=9u8 {
                if (candmask & (1<<d)) != 0 {
                    tried.push(d);
                    let mut child = grid.clone();
                    if let Err(_) = child.set(Pos{ r:i/9, c:i%9 }, d) { continue; }
                    logger.log(&format!("Search depth {}: try {} at r{},c{}", depth, d, i/9+1, i%9+1), &child.to_pretty_string())?;
                    if search(&mut child, logger, depth+1)? { *grid = child; return Ok(true); }
                }
            }
            logger.log(&format!("Backtrack depth {}", depth), &format!("Tried digits {:?} at r{},c{} without success", tried, i/9+1, i%9+1))?;
            Ok(false)
        }
        let solved = search(grid, logger, 0)?;
        if solved { Ok(()) } else { Ok(()) }
    }
}
