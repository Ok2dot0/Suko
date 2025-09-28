use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use suko::{grid::Grid, logger::DevLogger, solver::{Algorithm, SolveMode, Solver}};
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "suko", version, about = "Advanced Sudoku solver with devlogs")] 
struct Cli {
    /// Path to a puzzle file (81 chars with 0 or . for blanks). If omitted, reads from stdin.
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Choose solving approach
    #[arg(short, long, value_enum, default_value_t=Method::Hybrid)]
    method: Method,

    /// Step-by-step mode (pauses after each devlog step). Press Enter to continue.
    #[arg(long)]
    step: bool,

    /// Maximum devlogs to write (0 = unlimited)
    #[arg(long, default_value_t=0)]
    max_logs: usize,

    /// Emit devlogs to console with colors
    #[arg(long)]
    color: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Method { Logical, Search, Hybrid }

fn read_puzzle(input: &Option<PathBuf>) -> Result<String> {
    let s = match input {
        Some(p) => fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?,
        None => {
            use std::io::{self, Read};
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };
    let filtered: String = s.chars().filter(|ch| matches!(ch, '0'..='9'|'.')).collect();
    if filtered.len() < 81 { bail!("expected at least 81 digits/dots in input (have {})", filtered.len()) }
    Ok(filtered.chars().take(81).collect())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let puzzle = read_puzzle(&cli.input)?;
    let mut grid = Grid::from_compact(&puzzle).context("parse puzzle")?;
    let mut logger = DevLogger::new("devlogs", cli.color, cli.step, cli.max_logs)?;

    let mode = match cli.method {
        Method::Logical => SolveMode::LogicalOnly,
        Method::Search => SolveMode::SearchOnly,
        Method::Hybrid => SolveMode::Hybrid,
    };

    let mut solver = Solver::new(mode);
    solver.solve(&mut grid, &mut logger)?;

    println!("\nSolved grid:\n{}", grid.to_pretty_string());
    Ok(())
}
