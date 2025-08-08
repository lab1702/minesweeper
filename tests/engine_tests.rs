use minesweeper::engine::Board;

fn neighbors(w: usize, h: usize, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
    let x = x as isize; let y = y as isize; let w = w as isize; let h = h as isize;
    let mut out = Vec::new();
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 { continue; }
            let nx = x + dx; let ny = y + dy;
            if nx >= 0 && ny >= 0 && nx < w && ny < h { out.push((nx as usize, ny as usize)); }
        }
    }
    out.into_iter()
}

#[test]
fn safe_first_reveal_is_not_mine() {
    let mut b = Board::new(9, 9, 10, 12345).expect("board");
    let _ = b.reveal(0, 0);
    let c = b.cell(0, 0).unwrap();
    assert!(c.revealed());
    assert!(!c.is_mine());
}

#[test]
fn adjacency_matches_neighbor_mines() {
    let mut b = Board::new(8, 8, 10, 999).expect("board");
    let _ = b.reveal(0, 0); // initialize
    let w = b.width(); let h = b.height();
    let mut mine_count = 0;
    for y in 0..h {
        for x in 0..w {
            let c = b.cell(x, y).unwrap();
            if c.is_mine() { mine_count += 1; continue; }
            let mut adj = 0;
            for (nx, ny) in neighbors(w, h, x, y) {
                if b.cell(nx, ny).unwrap().is_mine() { adj += 1; }
            }
            assert_eq!(c.adjacent() as usize, adj, "adjacency mismatch at ({},{})", x, y);
        }
    }
    assert_eq!(mine_count, b.mines());
}

