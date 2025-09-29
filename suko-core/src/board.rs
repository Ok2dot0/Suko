use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub value: u8,        // 0 for empty
    pub fixed: bool,      // given by puzzle
}
a
impl Default for Cell {
    fn default() -> Self { Self { value: 0, fixed: false } }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub cells: [[Cell; 9]; 9],
}

impl Board {
    pub fn empty() -> Self { Self { cells: [[Cell::default(); 9]; 9] } }

    pub fn from_rows(rows: [[u8; 9]; 9]) -> Self {
        let mut b = Self::empty();
        for r in 0..9 { for c in 0..9 {
            let v = rows[r][c];
            b.cells[r][c] = Cell { value: v, fixed: v != 0 };
        }}
        b
    }

    pub fn parse(text: &str) -> anyhow::Result<Self> {
        // Accepts 81 characters of digits/./0 separated by whitespace/newlines
        let mut digits = Vec::with_capacity(81);
        for ch in text.chars() {
            match ch {
                '1'..='9' => digits.push(ch.to_digit(10).unwrap() as u8),
                '0' | '.' | '_' => digits.push(0),
                _ => { /* ignore other chars */ }
            }
        }
        if digits.len() != 81 { anyhow::bail!("expected 81 digits/dots, got {}", digits.len()); }
        let mut b = Self::empty();
        for r in 0..9 { for c in 0..9 { let idx = r*9+c; let v = digits[idx]; b.cells[r][c] = Cell { value: v, fixed: v!=0 }; }}
        Ok(b)
    }

    pub fn is_valid(&self) -> bool {
        // rows, cols, boxes have no duplicates ignoring zeros
        for r in 0..9 { if !no_dupes(self.row_values(r)) { return false; } }
        for c in 0..9 { if !no_dupes(self.col_values(c)) { return false; } }
        for br in 0..3 { for bc in 0..3 { if !no_dupes(self.box_values(br, bc)) { return false; } }}
        true
    }

    pub fn is_solved(&self) -> bool { self.cells.iter().all(|row| row.iter().all(|c| c.value != 0)) && self.is_valid() }

    pub fn row_values(&self, r: usize) -> [u8; 9] { let mut a=[0;9]; for c in 0..9 { a[c]=self.cells[r][c].value; } a }
    pub fn col_values(&self, c: usize) -> [u8; 9] { let mut a=[0;9]; for r in 0..9 { a[r]=self.cells[r][c].value; } a }
    pub fn box_values(&self, br: usize, bc: usize) -> [u8; 9] {
        let mut a=[0;9];
        let mut i=0;
        for r in br*3..br*3+3 { for c in bc*3..bc*3+3 { a[i]=self.cells[r][c].value; i+=1; }}
        a
    }

    pub fn candidates(&self, r: usize, c: usize) -> [bool; 10] {
        // index 1..=9 true if allowed
        if self.cells[r][c].value != 0 { let mut f=[false;10]; f[self.cells[r][c].value as usize]=true; return f; }
        let mut forb=[false;10];
        for x in self.row_values(r) { forb[x as usize]=true; }
        for x in self.col_values(c) { forb[x as usize]=true; }
        let br=r/3; let bc=c/3; for x in self.box_values(br,bc) { forb[x as usize]=true; }
        let mut cand=[false;10];
        for v in 1..=9 { cand[v as usize] = !forb[v as usize]; }
        cand
    }

    // Returns a mask of cells that are in conflict (duplicate non-zero values) in any row, column, or 3x3 box
    pub fn conflict_mask(&self) -> [[bool; 9]; 9] {
        let mut mask = [[false; 9]; 9];

        // Rows
        for r in 0..9 {
            let mut counts = [0u8; 10];
            for c in 0..9 {
                let v = self.cells[r][c].value as usize;
                if v != 0 { counts[v] += 1; }
            }
            for c in 0..9 {
                let v = self.cells[r][c].value as usize;
                if v != 0 && counts[v] > 1 { mask[r][c] = true; }
            }
        }

        // Columns
        for c in 0..9 {
            let mut counts = [0u8; 10];
            for r in 0..9 {
                let v = self.cells[r][c].value as usize;
                if v != 0 { counts[v] += 1; }
            }
            for r in 0..9 {
                let v = self.cells[r][c].value as usize;
                if v != 0 && counts[v] > 1 { mask[r][c] = true; }
            }
        }

        // Boxes
        for br in 0..3 {
            for bc in 0..3 {
                let mut counts = [0u8; 10];
                for r in br*3..br*3+3 {
                    for c in bc*3..bc*3+3 {
                        let v = self.cells[r][c].value as usize;
                        if v != 0 { counts[v] += 1; }
                    }
                }
                for r in br*3..br*3+3 {
                    for c in bc*3..bc*3+3 {
                        let v = self.cells[r][c].value as usize;
                        if v != 0 && counts[v] > 1 { mask[r][c] = true; }
                    }
                }
            }
        }

        mask
    }
}

fn no_dupes(vals: [u8;9]) -> bool {
    let mut seen=[false;10];
    for v in vals { if v!=0 { if seen[v as usize] { return false; } seen[v as usize]=true; }}
    true
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for r in 0..9 {
            for c in 0..9 {
                let v = self.cells[r][c].value;
                write!(f, "{}{}", if v==0 {'.'} else { char::from(b'0'+v) }, if c%3==2 && c!=8 {' '} else { ' ' })?;
            }
            if r%3==2 && r!=8 { writeln!(f)?; }
            writeln!(f)?;
        }
        Ok(())
    }
}
