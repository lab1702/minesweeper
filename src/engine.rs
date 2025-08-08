use std::fmt::{self, Write as _};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevealResult {
    NoOp,
    RevealedSafe,
    HitMine,
}

#[derive(Clone, Debug)]
pub struct Cell {
    is_mine: bool,
    adjacent: u8,
    revealed: bool,
    flagged: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self { is_mine: false, adjacent: 0, revealed: false, flagged: false }
    }
}

pub struct Board {
    width: usize,
    height: usize,
    mines: usize,
    cells: Vec<Cell>,
    remaining_safe: usize,
    alive: bool,
    won: bool,
    initialized: bool,
    seed: u64,
}

impl Board {
    pub fn new(width: usize, height: usize, mines: usize, mut seed: u64) -> Result<Self, String> {
        if width == 0 || height == 0 { return Err("Board dimensions must be positive".into()); }
        let total = width * height;
        if mines >= total { return Err("Mines must be less than cells".into()); }
        let mines = mines.min(total.saturating_sub(1));

        if seed == 0 { seed = seed_from_time(); }

        let cells = vec![Cell::default(); total];
        let remaining_safe = total - mines;

        Ok(Self { width, height, mines, cells, remaining_safe, alive: true, won: false, initialized: false, seed })
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height { return false; }
        let i = idx(self.width, x, y);
        if self.cells[i].revealed { return false; }
        self.cells[i].flagged = !self.cells[i].flagged;
        true
    }

    pub fn reveal(&mut self, x: usize, y: usize) -> RevealResult {
        if !self.alive || self.won { return RevealResult::NoOp; }
        if x >= self.width || y >= self.height { return RevealResult::NoOp; }
        let i = idx(self.width, x, y);
        if self.cells[i].flagged || self.cells[i].revealed { return RevealResult::NoOp; }

        if !self.initialized { self.initialize(x, y); }
        if self.cells[i].is_mine { self.alive = false; return RevealResult::HitMine; }

        // Flood-fill reveal when adjacent == 0
        self.flood_reveal(x, y);
        if self.remaining_safe == 0 && self.alive {
            self.won = true;
        }
        RevealResult::RevealedSafe
    }

    fn initialize(&mut self, safe_x: usize, safe_y: usize) {
        if self.initialized { return; }
        let total = self.width * self.height;
        let safe_idx = idx(self.width, safe_x, safe_y);
        let mut positions: Vec<usize> = (0..total).filter(|&p| p != safe_idx).collect();
        let mut prng = XorShift64::new(self.seed);
        fisher_yates_shuffle(&mut positions, &mut prng);
        for &pos in &positions[..self.mines] {
            self.cells[pos].is_mine = true;
        }
        self.compute_adjacency();
        self.initialized = true;
    }

    fn compute_adjacency(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let i0 = idx(self.width, x, y);
                if self.cells[i0].is_mine { continue; }
                let mut c = 0u8;
                for (nx, ny) in neighbors(self.width, self.height, x, y) {
                    if self.cells[idx(self.width, nx, ny)].is_mine { c += 1; }
                }
                self.cells[i0].adjacent = c;
            }
        }
    }

    fn flood_reveal(&mut self, x: usize, y: usize) {
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            let i = idx(self.width, cx, cy);
            if self.cells[i].revealed || self.cells[i].flagged { continue; }
            if self.cells[i].is_mine { continue; }
            self.cells[i].revealed = true;
            if self.remaining_safe > 0 { self.remaining_safe -= 1; }
            if self.cells[i].adjacent == 0 {
                for (nx, ny) in neighbors(self.width, self.height, cx, cy) {
                    let ni = idx(self.width, nx, ny);
                    if !self.cells[ni].revealed && !self.cells[ni].is_mine {
                        stack.push((nx, ny));
                    }
                }
            }
        }
    }

    pub fn render(&self, show_all: bool, one_based: bool) -> String {
        let mut s = String::new();
        // Column header
        s.push_str("    ");
        for x in 0..self.width {
            let label = if one_based { x + 1 } else { x };
            let _ = write!(s, "{:>2} ", label);
        }
        s.push('\n');
        s.push_str("   ");
        s.push_str(&"-".repeat(self.width * 3 + 1));
        s.push('\n');

        for y in 0..self.height {
            let row_label = if one_based { y + 1 } else { y };
            let _ = write!(s, "{:>2} | ", row_label);
            for x in 0..self.width {
                let c = &self.cells[idx(self.width, x, y)];
                let ch = if show_all && c.is_mine {
                    '*'
                } else if c.revealed {
                    if c.is_mine { '*' } else if c.adjacent == 0 { ' ' } else { char::from_digit(c.adjacent as u32, 10).unwrap_or('?') }
                } else if c.flagged {
                    'F'
                } else {
                    '.'
                };
                let _ = write!(s, "{}  ", ch);
            }
            s.push('\n');
        }
        s
    }
}

fn idx(w: usize, x: usize, y: usize) -> usize { y * w + x }

fn neighbors(w: usize, h: usize, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
    let x = x as isize; let y = y as isize; let w = w as isize; let h = h as isize;
    let mut out = Vec::with_capacity(8);
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 { continue; }
            let nx = x + dx; let ny = y + dy;
            if nx >= 0 && ny >= 0 && nx < w && ny < h {
                out.push((nx as usize, ny as usize));
            }
        }
    }
    out.into_iter()
}

// Simple xorshift64 PRNG to avoid external dependencies.
struct XorShift64 { state: u64 }
impl XorShift64 {
    fn new(seed: u64) -> Self { Self { state: seed.max(1) } }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    fn next_usize(&mut self) -> usize { (self.next_u64() >> 1) as usize }
}

fn fisher_yates_shuffle<T>(arr: &mut [T], prng: &mut XorShift64) {
    // Standard FY: for i from n-1 down to 1, swap i with random j in [0, i]
    let n = arr.len();
    for i in (1..n).rev() {
        let j = prng.next_usize() % (i + 1);
        arr.swap(i, j);
    }
}

fn seed_from_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    now.as_nanos() as u64 ^ (now.as_secs() as u64).rotate_left(32)
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render(false, true))
    }
}

// Public getters for encapsulation
impl Board {
    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn mines(&self) -> usize { self.mines }
    pub fn remaining_safe(&self) -> usize { self.remaining_safe }
    pub fn alive(&self) -> bool { self.alive }
    pub fn won(&self) -> bool { self.won }
    pub fn cell(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height { Some(&self.cells[idx(self.width, x, y)]) } else { None }
    }
}

impl Cell {
    pub fn is_mine(&self) -> bool { self.is_mine }
    pub fn adjacent(&self) -> u8 { self.adjacent }
    pub fn revealed(&self) -> bool { self.revealed }
    pub fn flagged(&self) -> bool { self.flagged }
}
