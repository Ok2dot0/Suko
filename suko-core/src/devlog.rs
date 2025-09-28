use time::macros::format_description;
use time::OffsetDateTime;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use crate::solver::Step;

pub struct DevLogger {
    root: PathBuf,
    index: usize,
}

impl DevLogger {
    pub fn new(root: impl Into<PathBuf>) -> std::io::Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        // Determine next index by scanning existing files
        let mut max_idx = 0usize;
        if let Ok(rd) = fs::read_dir(&root) { for e in rd.flatten() { if let Some(name)=e.file_name().to_str() {
            if let Some(num) = name.strip_prefix("devlog").and_then(|s| s.strip_suffix(".txt")).and_then(|n| n.parse::<usize>().ok()) { if num>max_idx { max_idx=num; } }
        }}}
        Ok(Self { root, index: max_idx })
    }

    pub fn next_file(&mut self) -> PathBuf {
        self.index += 1;
        self.root.join(format!("devlog{}.txt", self.index))
    }

    pub fn write_log(&mut self, title: &str, lines: &[impl AsRef<str>]) -> std::io::Result<PathBuf> {
        let path = self.next_file();
        let mut f = OpenOptions::new().create(true).write(true).open(&path)?;
        let ts_fmt = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        let now = OffsetDateTime::now_utc().format(&ts_fmt).unwrap_or_else(|_| "unknown".into());
        writeln!(f, "{}", title)?;
        writeln!(f, "Timestamp: {} UTC", now)?;
        writeln!(f, "----------------------------------------")?;
        for l in lines { writeln!(f, "{}", l.as_ref())?; }
        Ok(path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    pub title: String,
    pub puzzle: String,
    pub solver_name: String,
    pub steps: Vec<Step>,
}

pub fn write_session_markdown<P: AsRef<Path>>(dir: P, log: &SessionLog) -> std::io::Result<PathBuf> {
    fs::create_dir_all(dir.as_ref())?;
    let ts_fmt = format_description!("[year]-[month]-[day]_[hour][minute][second]");
    let now = OffsetDateTime::now_utc().format(&ts_fmt).unwrap_or_else(|_| "now".into());
    let filename = format!("session_{}_.md", now);
    let path = dir.as_ref().join(filename);
    let mut f = OpenOptions::new().create(true).write(true).open(&path)?;
    writeln!(f, "# {}", log.title)?;
    writeln!(f, "Solver: {}", log.solver_name)?;
    writeln!(f, "Puzzle: `{}`", log.puzzle)?;
    writeln!(f, "\n## Steps")?;
    for s in &log.steps {
        writeln!(f, "\n### Step {}", s.index)?;
        match &s.kind { 
            crate::solver::StepKind::Place{ r,c,v,reason } => writeln!(f, "- Place {} at ({}, {}) — {}", v, r+1, c+1, reason)?,
            crate::solver::StepKind::Guess{ r,c,v } => writeln!(f, "- Guess {} at ({}, {})", v, r+1, c+1)?,
            crate::solver::StepKind::Backtrack => writeln!(f, "- Backtrack")?,
        }
        writeln!(f, "\n``\n{}\n``", s.board)?;
    }
    Ok(path)
}
