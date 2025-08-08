Minesweeper (Rust CLI)
======================

Simple, dependency-free Minesweeper playable in the terminal.

Build and Run
-------------

- Requirements: Rust toolchain (edition 2021).

Build:

```
cargo build --release
```

Run with defaults (9x9, 10 mines, random seed):

```
cargo run --release
```

Custom size/mines (flags):

```
cargo run --release -- --width 16 --height 16 --mines 40
cargo run --release -- --width 30 --height 16 --mines 99 --seed 12345
```

TUI Mode
--------

Launch the interactive terminal UI (arrow keys, space/enter to reveal, `f` to flag):

```
cargo run --release -- --tui --width 16 --height 16 --mines 40
```

Controls: arrows/HJKL to move, Enter/Space reveal, `f` flag, `n` new game, `q` quit.

Non-interactive demo (for CI/headless runs):

```
MINESWEEPER_TUI_AUTODEMO=1 cargo run --release -- --tui --width 9 --height 9 --mines 10 --seed 42
```

Gameplay
--------

- Coordinates are 1-based (column x, row y).
- Commands:
  - `r x y`: reveal cell
  - `f x y`: toggle flag
  - `q`: quit
  - `h` / `help`: show help

Display
-------

- `.`: covered cell
- `F`: flagged cell
- ` ` (space): revealed empty (0 adjacent mines)
- `1`..`8`: revealed with adjacent mine count
- `*`: mine (revealed at game end)

Notes
-----

- The game uses an internal xorshift PRNG; specify a non-zero seed for reproducible boards.
- First move is always safe: mines are placed after your first reveal, excluding that cell.
