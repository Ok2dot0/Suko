use anyhow::Result;

use crate::{grid::{Grid, Pos, bitcount, first_bit}, logger::DevLogger};

pub mod strategies {
    use super::*;

    pub fn naked_single(grid: &mut Grid, logger: &mut DevLogger) -> Result<bool> {
        for i in 0..81 { if grid.cells[i]==0 {
            let m = grid.cands[i];
            if bitcount(m)==1 { let d = (first_bit(m).unwrap()+1) as u8; grid.set(Pos{r:i/9,c:i%9}, d)?; logger.log("Naked single", &format!("Placed {} at r{},c{}", d, i/9+1, i%9+1))?; return Ok(true); }
        }}
        Ok(false)
    }

    pub fn hidden_single(grid: &mut Grid, logger: &mut DevLogger) -> Result<bool> {
        // check each unit (row, col, box) for a digit that appears in exactly one candidate
        // rows
        for r in 0..9 {
            for d in 1..=9u8 {
                let mut where_i = None; let mut cnt=0;
                for c in 0..9 { let i=r*9+c; if grid.cells[i]==0 && (grid.cands[i] & (1<<d))!=0 { cnt+=1; where_i=Some(i);} }
                if cnt==1 { let i = where_i.unwrap(); grid.set(Pos{r:i/9,c:i%9}, d)?; logger.log("Hidden single (row)", &format!("Placed {} at r{},c{}", d, r+1, i%9+1))?; return Ok(true); }
            }
        }
        // cols
        for c in 0..9 {
            for d in 1..=9u8 {
                let mut where_i = None; let mut cnt=0;
                for r in 0..9 { let i=r*9+c; if grid.cells[i]==0 && (grid.cands[i] & (1<<d))!=0 { cnt+=1; where_i=Some(i);} }
                if cnt==1 { let i = where_i.unwrap(); grid.set(Pos{r:i/9,c:i%9}, d)?; logger.log("Hidden single (col)", &format!("Placed {} at r{},c{}", d, i/9+1, c+1))?; return Ok(true); }
            }
        }
        // boxes
        for br in (0..9).step_by(3) { for bc in (0..9).step_by(3) {
            for d in 1..=9u8 {
                let mut where_i=None; let mut cnt=0;
                for rr in br..br+3 { for cc in bc..bc+3 { let i=rr*9+cc; if grid.cells[i]==0 && (grid.cands[i]&(1<<d))!=0 { cnt+=1; where_i=Some(i);} }}
                if cnt==1 { let i = where_i.unwrap(); grid.set(Pos{r:i/9,c:i%9}, d)?; logger.log("Hidden single (box)", &format!("Placed {} at r{},c{}", d, i/9+1, i%9+1))?; return Ok(true); }
            }
        }}
        Ok(false)
    }

    pub fn naked_pair(grid: &mut Grid, logger: &mut DevLogger) -> Result<bool> {
        // check each row
        let mut progress=false;
        // helper closure
        let mut process_unit = |indices: Vec<usize>| -> Result<bool> {
            let mut found=false;
            for &i in &indices { if grid.cells[i]==0 { let m=grid.cands[i]; if bitcount(m)==2 {
                for &j in &indices { if i!=j && grid.cells[j]==0 && grid.cands[j]==m {
                    // eliminate m from others in unit
                    for &k in &indices { if k!=i && k!=j && grid.cells[k]==0 {
                        let before = grid.cands[k];
                        let after = before & !m;
                        if after!=before { grid.cands[k]=after; found=true; }
                    }}
                    if found { logger.log("Naked pair", &format!("Pair {:?} at cells {:?} in unit eliminated from others", mask_to_digits(m), (i,j)))?; return Ok(true); }
                }} }}
            Ok(false)
        };
        for r in 0..9 { if process_unit((0..9).map(|c| r*9+c).collect())? { progress=true; break; } }
        if !progress { for c in 0..9 { if process_unit((0..9).map(|r| r*9+c).collect())? { progress=true; break; } } }
        if !progress { for br in (0..9).step_by(3) { for bc in (0..9).step_by(3) { let mut v=Vec::new(); for rr in br..br+3 { for cc in bc..bc+3 { v.push(rr*9+cc); } } if process_unit(v)? { progress=true; break; } }} }
        Ok(progress)
    }

    pub fn pointing_pair_triple(grid: &mut Grid, logger: &mut DevLogger) -> Result<bool> {
        // For each box and digit, if candidates lie in only one row or one column within the box, eliminate from that row/col outside the box
        for br in (0..9).step_by(3) { for bc in (0..9).step_by(3) {
            for d in 1..=9u8 {
                let mut rows = [0u8;3];
                let mut cols = [0u8;3];
                for rr in 0..3 { for cc in 0..3 { let i=(br+rr)*9 + (bc+cc); if grid.cells[i]==0 && (grid.cands[i]&(1<<d))!=0 { rows[rr]+=1; cols[cc]+=1; } }}
                if rows.iter().filter(|&&x| x>0).count()==1 {
                    let rr = rows.iter().position(|&x| x>0).unwrap();
                    let target_r = br+rr; let mut changed=false;
                    for c in 0..9 { if c<bc || c>=bc+3 { let i=target_r*9+c; if grid.cells[i]==0 { let before=grid.cands[i]; let after=before & !(1<<d); if after!=before { grid.cands[i]=after; changed=true; } } }}
                    if changed { logger.log("Pointing pair/triple (row)", &format!("Digit {} confined to row {} in box {},{}; eliminated from row", d, target_r+1, br/3+1, bc/3+1))?; return Ok(true); }
                }
                if cols.iter().filter(|&&x| x>0).count()==1 {
                    let cc = cols.iter().position(|&x| x>0).unwrap();
                    let target_c = bc+cc; let mut changed=false;
                    for r in 0..9 { if r<br || r>=br+3 { let i=r*9+target_c; if grid.cells[i]==0 { let before=grid.cands[i]; let after=before & !(1<<d); if after!=before { grid.cands[i]=after; changed=true; } } }}
                    if changed { logger.log("Pointing pair/triple (col)", &format!("Digit {} confined to col {} in box {},{}; eliminated from column", d, target_c+1, br/3+1, bc/3+1))?; return Ok(true); }
                }
            }
        }}
        Ok(false)
    }

    pub fn box_line_reduction(grid: &mut Grid, logger: &mut DevLogger) -> Result<bool> {
        // For each row and digit, if candidates lie only in one box, eliminate from that box outside the row; similarly for columns
        // rows
        for r in 0..9 { for d in 1..=9u8 {
            let mut boxes = [0u8;3];
            for c in 0..9 { let i=r*9+c; if grid.cells[i]==0 && (grid.cands[i]&(1<<d))!=0 { boxes[c/3]+=1; } }
            if boxes.iter().filter(|&&x| x>0).count()==1 { let b = boxes.iter().position(|&x| x>0).unwrap(); let bc=b*3; let mut changed=false; for rr in (r/3)*3..(r/3)*3+3 { for cc in bc..bc+3 { let i=rr*9+cc; if rr!=r && grid.cells[i]==0 { let before=grid.cands[i]; let after=before & !(1<<d); if after!=before { grid.cands[i]=after; changed=true; } } }} if changed { logger.log("Box-Line reduction (row)", &format!("Digit {} in row {} confined to box {}; eliminated from box", d, r+1, b+1))?; return Ok(true);} }
        }}
        // cols
        for c in 0..9 { for d in 1..=9u8 {
            let mut boxes = [0u8;3];
            for r in 0..9 { let i=r*9+c; if grid.cells[i]==0 && (grid.cands[i]&(1<<d))!=0 { boxes[r/3]+=1; } }
            if boxes.iter().filter(|&&x| x>0).count()==1 { let b = boxes.iter().position(|&x| x>0).unwrap(); let br=b*3; let mut changed=false; for rr in br..br+3 { for cc in (c/3)*3..(c/3)*3+3 { let i=rr*9+cc; if cc!=c && grid.cells[i]==0 { let before=grid.cands[i]; let after=before & !(1<<d); if after!=before { grid.cands[i]=after; changed=true; } } }} if changed { logger.log("Box-Line reduction (col)", &format!("Digit {} in col {} confined to box {}; eliminated from box", d, c+1, b+1))?; return Ok(true);} }
        }}
        Ok(false)
    }

    pub fn x_wing(grid: &mut Grid, logger: &mut DevLogger) -> Result<bool> {
        // Basic X-Wing on rows and columns for each digit
        for d in 1..=9u8 {
            // rows
            let mut row_cols: Vec<(usize, Vec<usize>)> = Vec::new();
            for r in 0..9 { let positions: Vec<usize> = (0..9).filter(|&c| grid.cells[r*9+c]==0 && (grid.cands[r*9+c]&(1<<d))!=0).collect(); if positions.len()==2 { row_cols.push((r, positions)); } }
            for i in 0..row_cols.len() { for j in i+1..row_cols.len() { let (r1, ref c1)=row_cols[i]; let (r2, ref c2)=row_cols[j]; if c1==c2 { let mut changed=false; for &c in c1 { for r in 0..9 { if r!=r1 && r!=r2 { let idx=r*9+c; if grid.cells[idx]==0 { let before=grid.cands[idx]; let after=before & !(1<<d); if after!=before { grid.cands[idx]=after; changed=true; } } } } } if changed { logger.log("X-Wing (rows)", &format!("Digit {} with rows {} and {} at cols {:?}", d, r1+1, r2+1, c1.iter().map(|x| x+1).collect::<Vec<_>>()))?; return Ok(true); } }}
            // cols
            let mut col_rows: Vec<(usize, Vec<usize>)> = Vec::new();
            for c in 0..9 { let positions: Vec<usize> = (0..9).filter(|&r| grid.cells[r*9+c]==0 && (grid.cands[r*9+c]&(1<<d))!=0).collect(); if positions.len()==2 { col_rows.push((c, positions)); } }
            for i in 0..col_rows.len() { for j in i+1..col_rows.len() { let (c1, ref r1s)=col_rows[i]; let (c2, ref r2s)=col_rows[j]; if r1s==r2s { let mut changed=false; for &r in r1s { for c in 0..9 { if c!=c1 && c!=c2 { let idx=r*9+c; if grid.cells[idx]==0 { let before=grid.cands[idx]; let after=before & !(1<<d); if after!=before { grid.cands[idx]=after; changed=true; } } } } } if changed { logger.log("X-Wing (cols)", &format!("Digit {} with cols {} and {} at rows {:?}", d, c1+1, c2+1, r1s.iter().map(|x| x+1).collect::<Vec<_>>()))?; return Ok(true); } }}
        }
        Ok(false)
    }

    fn mask_to_digits(m: u16) -> Vec<u8> { (1..=9).filter(|&d| (m & (1<<d))!=0).collect() }
}
