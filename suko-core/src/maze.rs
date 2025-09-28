use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

#[derive(Clone, Copy)]
struct Cell { visited: bool, walls: [bool;4] } // 0:Up,1:Right,2:Down,3:Left

pub struct Maze {
    pub width: usize,
    pub height: usize,
    grid: Vec<Cell>,
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![Cell{ visited:false, walls:[true,true,true,true]}; width*height];
        Self{ width, height, grid }
    }

    fn idx(&self, x: usize, y: usize) -> usize { y*self.width + x }

    pub fn generate_recursive_backtracker(width: usize, height: usize, seed: Option<u64>) -> Self {
        let mut maze = Self::new(width, height);
        let mut rng = match seed { Some(s) => rand::rngs::StdRng::seed_from_u64(s), None => rand::rngs::StdRng::from_rng(rand::thread_rng()).unwrap() };
        let mut stack: Vec<(usize,usize)> = Vec::new();
        let sx = rng.gen_range(0..width); let sy = rng.gen_range(0..height);
        stack.push((sx,sy));
    let start_idx = maze.idx(sx, sy);
    maze.grid[start_idx].visited = true;

        while let Some(&(cx,cy)) = stack.last() {
            let mut neighbors = Vec::new();
            if cy>0 && !maze.grid[maze.idx(cx,cy-1)].visited { neighbors.push((cx,cy-1,0)); }
            if cx+1<width && !maze.grid[maze.idx(cx+1,cy)].visited { neighbors.push((cx+1,cy,1)); }
            if cy+1<height && !maze.grid[maze.idx(cx,cy+1)].visited { neighbors.push((cx,cy+1,2)); }
            if cx>0 && !maze.grid[maze.idx(cx-1,cy)].visited { neighbors.push((cx-1,cy,3)); }

            if neighbors.is_empty() { stack.pop(); continue; }
            neighbors.shuffle(&mut rng);
            let (nx,ny,dir) = neighbors[0];
            // carve between (cx,cy) and (nx,ny)
            let opp = (dir+2)%4;
            let cur_idx = maze.idx(cx, cy);
            let next_idx = maze.idx(nx, ny);
            maze.grid[cur_idx].walls[dir as usize] = false;
            maze.grid[next_idx].walls[opp as usize] = false;
            maze.grid[next_idx].visited = true;
            stack.push((nx,ny));
        }
        maze
    }

    pub fn to_ascii(&self) -> String {
        // Each cell -> 2x1 chars horizontally for walls; draw top border and rows
        let mut s = String::new();
        // top border
        s.push('+');
        for _x in 0..self.width { s.push_str("--+"); }
        s.push('\n');
        for y in 0..self.height {
            // left wall and cells
            let mut line1 = String::from("|");
            let mut line2 = String::from("+");
            for x in 0..self.width {
                let c = self.grid[self.idx(x,y)];
                line1.push_str("  ");
                line1.push(if c.walls[1] { '|' } else { ' ' });
                line2.push_str(if c.walls[2] { "--" } else { "  " });
                line2.push('+');
            }
            s.push_str(&line1); s.push('\n');
            s.push_str(&line2); s.push('\n');
        }
        s
    }
}
