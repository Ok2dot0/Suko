use anyhow::{bail, Result};
use itertools::Itertools;

pub type Digit = u8; // 1..=9

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos { pub r: usize, pub c: usize }

impl Pos { pub fn idx(self) -> usize { self.r * 9 + self.c } }

#[derive(Clone, Debug)]
pub struct Grid {
    // 0 = empty; 1..=9 digits
    pub(crate) cells: [Digit; 81],
    // candidate bitset per cell; bit d means digit d (1..=9) possible
    pub(crate) cands: [u16; 81],
}

impl Grid {
    pub fn empty() -> Self { Self { cells: [0;81], cands: [all_candidates();81] } }

    pub fn from_compact(s: &str) -> Result<Self> {
        if s.len() != 81 { bail!("compact string must be 81 chars") }
        let mut g = Grid::empty();
        for (i, ch) in s.chars().enumerate() {
            let v = match ch { '.'|'0' => 0, '1'..='9' => ch as u8 - b'0', _ => bail!("invalid char {ch}") };
            if v != 0 { g.place_idx(i, v)?; }
        }
        Ok(g)
    }

    pub fn to_compact(&self) -> String {
        self.cells.iter().map(|&d| if d==0 {'.'} else {(b'0'+d) as char}).collect()
    }

    pub fn to_pretty_string(&self) -> String {
        let mut s = String::new();
        for r in 0..9 {
            if r%3==0 { s.push_str("+-------+-------+-------+\n"); }
            for c in 0..9 {
                if c%3==0 { s.push('|'); s.push(' ');}    
                let d = self.get(Pos{r,c});
                s.push(if d==0 {'·'} else {(b'0'+d) as char});
                s.push(' ');
            }
            s.push('|'); s.push('\n');
        }
        s.push_str("+-------+-------+-------+\n");
        s
    }

    pub fn get(&self, p: Pos) -> Digit { self.cells[p.idx()] }
    pub fn candidates(&self, p: Pos) -> u16 { self.cands[p.idx()] }
    pub fn is_solved(&self) -> bool { self.cells.iter().all(|&d| d!=0) }

    pub fn set(&mut self, p: Pos, d: Digit) -> Result<()> { self.place_idx(p.idx(), d) }

    fn place_idx(&mut self, idx: usize, d: Digit) -> Result<()> {
        if d<1 || d>9 { bail!("digit out of range") }
        if self.cells[idx] == d { return Ok(()); }
        if self.cells[idx] != 0 { bail!("cell already filled") }
        if self.cands[idx] & (1 << d) == 0 { bail!("candidate {d} not allowed here") }
        self.cells[idx] = d;
        self.cands[idx] = 0;
        // eliminate d from peers
        for q in peers_of_idx(idx) {
            self.cands[q] &= !(1 << d);
            if self.cells[q]==0 && self.cands[q]==0 { bail!("contradiction: no candidates left") }
        }
        Ok(())
    }

    pub fn infer_all_candidates(&mut self) -> Result<()> {
        // reset candidates based on current assignments
        self.cands = [all_candidates();81];
        for idx in 0..81 { if self.cells[idx]!=0 { self.cands[idx]=0; } }
        for idx in 0..81 { let d = self.cells[idx]; if d!=0 { for q in peers_of_idx(idx) { self.cands[q] &= !(1<<d); } } }
        // check contradictions
        for idx in 0..81 { if self.cells[idx]==0 && self.cands[idx]==0 { bail!("no candidates at {}", idx) } }
        Ok(())
    }

    pub fn iterate_cells() -> impl Iterator<Item=Pos> { (0..81).map(|i| Pos{ r:i/9, c:i%9 }) }
}

pub fn peers_of(p: Pos) -> Vec<usize> { peers_of_idx(p.idx()) }

fn peers_of_idx(idx: usize) -> Vec<usize> {
    let r = idx/9; let c = idx%9;
    let br = (r/3)*3; let bc = (c/3)*3;
    let mut v = Vec::with_capacity(20);
    for i in 0..9 { if i!=c { v.push(r*9+i);} }
    for i in 0..9 { if i!=r { v.push(i*9+c);} }
    for rr in br..br+3 { for cc in bc..bc+3 { let j = rr*9+cc; if rr!=r || cc!=c { v.push(j); } } }
    v.sort_unstable(); v.dedup();
    v
}

pub fn bitcount(x: u16) -> u32 { x.count_ones() }
pub fn first_bit(x: u16) -> Option<u8> { if x==0 {None} else { Some(x.trailing_zeros() as u8) } }

#[inline]
pub const fn all_candidates() -> u16 { 0b11_1111_1110 } // bits 1..=9 set (1022)
