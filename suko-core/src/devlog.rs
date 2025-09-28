use time::macros::format_description;
use time::OffsetDateTime;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

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
