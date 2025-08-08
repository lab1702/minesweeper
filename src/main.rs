use std::io::{self, Write};

use clap::Parser;
use minesweeper::engine::{Board, RevealResult};
use minesweeper::tui;

#[derive(Parser, Debug)]
#[command(name = "minesweeper", about = "Rust CLI/TUI Minesweeper", version)]
struct Args {
    /// Launch TUI mode
    #[arg(long)]
    tui: bool,
    /// Board width
    #[arg(long, default_value_t = 9)]
    width: usize,
    /// Board height
    #[arg(long, default_value_t = 9)]
    height: usize,
    /// Number of mines
    #[arg(long, default_value_t = 10)]
    mines: usize,
    /// Seed (0 = random)
    #[arg(long, default_value_t = 0)]
    seed: u64,
}

fn print_help() {
    println!("Commands:");
    println!("  r x y   - reveal cell at column x, row y (1-based)");
    println!("  f x y   - toggle flag at x, y (1-based)");
    println!("  q       - quit");
    println!("  h/help  - show this help");
}

fn main() {
    let args = Args::parse();
    if args.tui {
        if let Err(e) = tui::run_tui(args.width, args.height, args.mines, args.seed) {
            eprintln!("TUI error: {}", e);
        }
        return;
    }
    let mut board = match Board::new(args.width, args.height, args.mines, args.seed) {
        Ok(b) => b,
        Err(e) => { eprintln!("{}", e); return; }
    };

    println!("Minesweeper {}x{} with {} mines{}", args.width, args.height, args.mines, if args.seed != 0 { format!(" (seed {})", args.seed) } else { String::new() });
    println!("Coordinates are 1-based. Type 'h' for help.");
    print_help();

    let mut input = String::new();
    loop {
        println!("\n{}", board);
        if !board.alive() {
            println!("Boom! You hit a mine. Game over.\n");
            println!("Final board (mines shown):\n{}", board.render(true, true));
            break;
        }
        if board.won() {
            println!("Congratulations! You cleared the board!\n");
            println!("Final board (mines shown):\n{}", board.render(true, true));
            break;
        }

        print!("> ");
        let _ = io::stdout().flush();
        input.clear();
        if io::stdin().read_line(&mut input).is_err() { break; }
        let line = input.trim();
        if line.is_empty() { continue; }

        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0].to_lowercase().as_str() {
            "q" | "quit" | "exit" => break,
            "h" | "help" => { print_help(); continue; },
            "r" | "reveal" => {
                if parts.len() < 3 { println!("Usage: r x y"); continue; }
                let x = match parts[1].parse::<usize>() { Ok(v) => v, Err(_) => { println!("Invalid x"); continue; } };
                let y = match parts[2].parse::<usize>() { Ok(v) => v, Err(_) => { println!("Invalid y"); continue; } };
                if x == 0 || y == 0 { println!("Use 1-based coordinates"); continue; }
                let res = board.reveal(x-1, y-1);
                match res {
                    RevealResult::HitMine => { /* handled at loop top */ },
                    RevealResult::RevealedSafe => { /* ok */ },
                    RevealResult::NoOp => { /* ignore */ },
                }
            }
            "f" | "flag" => {
                if parts.len() < 3 { println!("Usage: f x y"); continue; }
                let x = match parts[1].parse::<usize>() { Ok(v) => v, Err(_) => { println!("Invalid x"); continue; } };
                let y = match parts[2].parse::<usize>() { Ok(v) => v, Err(_) => { println!("Invalid y"); continue; } };
                if x == 0 || y == 0 { println!("Use 1-based coordinates"); continue; }
                if !board.toggle_flag(x-1, y-1) { println!("Cannot flag revealed cell or out of bounds"); }
            }
            other => {
                println!("Unknown command '{}'. Type 'h' for help.", other);
            }
        }
    }
}
