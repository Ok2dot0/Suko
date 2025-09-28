use suko::{Grid, Solver, SolveMode, logger::DevLogger};

#[test]
fn parse_and_format() {
    let s = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    let g = Grid::from_compact(s).unwrap();
    assert_eq!(g.to_compact().len(), 81);
}

#[test]
fn solve_easy_hybrid() {
    let s = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    let mut g = Grid::from_compact(s).unwrap();
    let mut logger = DevLogger::new("devlogs_test", false, false, 50).unwrap();
    let mut solver = Solver::new(SolveMode::Hybrid);
    solver.solve(&mut g, &mut logger).unwrap();
    assert!(g.is_solved());
}
