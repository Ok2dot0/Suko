use anyhow::Result;
use chrono::Local;
use colored::*;
use std::{fs::{self, File}, io::{Write, Read}, path::PathBuf};

pub struct DevLogger {
    dir: PathBuf,
    color: bool,
    step: bool,
    max_logs: usize,
    counter: usize,
}

impl DevLogger {
    pub fn new(dir: impl Into<PathBuf>, color: bool, step: bool, max_logs: usize) -> Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self { dir, color, step, max_logs, counter: 0 })
    }

    pub fn log(&mut self, title: &str, details: &str) -> Result<()> {
        if self.max_logs != 0 && self.counter >= self.max_logs { return Ok(()); }
        self.counter += 1;
        let filename = format!("devlog({}).txt", self.counter);
        let mut path = self.dir.clone();
        path.push(filename);

        let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
        let mut f = File::create(&path)?;
        writeln!(f, "[{}] {}\n\n{}", ts, title, details)?;

        if self.color {
            println!("{} {}\n{}", "➤".blue().bold(), title.bold(), details);
        } else {
            println!("➤ {}\n{}", title, details);
        }

        if self.step {
            print!("-- press Enter to continue --");
            use std::io::{self, Write as _};
            io::stdout().flush().ok();
            let mut s = String::new();
            io::stdin().read_line(&mut s).ok();
        }
        Ok(())
    }
}
