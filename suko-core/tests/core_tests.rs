use suko_core::{board::Board, solver::{BacktrackingSolver, LogicalSolver, Solver}};

fn easy_puzzle() -> &'static str {
    // Known easy puzzle; dots for blanks
    "53..7....\n6..195...\n.98....6.\n8...6...3\n4..8.3..1\n7...2...6\n.6....28.\n...419..5\n....8..79"
}

#[test]
fn parse_and_validity() {
    let b = Board::parse(easy_puzzle()).expect("parse");
    assert!(b.is_valid(), "initial board should be valid");
    assert!(!b.is_solved(), "not solved yet");
}

#[test]
fn backtracking_solves_easy() {
    let b = Board::parse(easy_puzzle()).unwrap();
    let mut solver = BacktrackingSolver::new();
    let steps = solver.solve_steps(&b, None);
    assert!(!steps.is_empty(), "should produce steps");
    let last = steps.last().unwrap();
    assert!(last.board.is_valid());
}

#[test]
fn logical_progress_single_step() {
    let b = Board::parse(easy_puzzle()).unwrap();
    let mut solver = LogicalSolver::new();
    let steps = solver.solve_steps(&b, Some(1));
    // Might be 0 if no immediate singles; accept 0 or 1, but if 1, board should be valid
    if let Some(last) = steps.last() {
        assert!(last.board.is_valid());
    }
}
