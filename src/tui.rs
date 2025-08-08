use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

use crate::engine::Board;

pub fn run_tui(width: usize, height: usize, mines: usize, seed: u64) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableMouseCapture)?;
    let _guard = TermGuard;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut board = Board::new(width, height, mines, seed).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut cursor = (0usize, 0usize);
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    let autodemo = std::env::var("MINESWEEPER_TUI_AUTODEMO").ok().is_some();
    let mut demo_step = 0usize;

    let mut last_inner_board = Rect::default();
    let res = loop {
        terminal.draw(|f| { last_inner_board = ui(f, &board, cursor); })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                        KeyCode::Char('h') | KeyCode::Left => {
                            if cursor.0 > 0 { cursor.0 -= 1; }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            if cursor.0 + 1 < board.width() { cursor.0 += 1; }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if cursor.1 > 0 { cursor.1 -= 1; }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if cursor.1 + 1 < board.height() { cursor.1 += 1; }
                        }
                        KeyCode::Char('f') => { let _ = board.toggle_flag(cursor.0, cursor.1); }
                        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('r') => {
                            let _ = board.reveal(cursor.0, cursor.1);
                        }
                        KeyCode::Char('n') => { if let Ok(b) = Board::new(width, height, mines, seed) { board = b; } }
                        KeyCode::Char('R') if shift => { if let Ok(b) = Board::new(width, height, mines, seed) { board = b; } }
                        _ => {}
                    }
                }
                Event::Mouse(m) => {
                    // Map mouse to cell coordinates within the inner board area
                    if let MouseEventKind::Down(btn) = m.kind {
                        if let Some((cx, cy)) = pos_to_cell(m.column, m.row, last_inner_board, board.width() as u16, board.height() as u16) {
                            match btn {
                                MouseButton::Left => { let _ = board.reveal(cx as usize, cy as usize); }
                                MouseButton::Right => { let _ = board.toggle_flag(cx as usize, cy as usize); }
                                MouseButton::Middle => { /* reserved for future chording */ }
                            }
                        }
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            if autodemo {
                // simple scripted steps then exit
                match demo_step {
                    0 => { let _ = board.reveal(0, 0); cursor = (1.min(board.width()-1), 1.min(board.height()-1)); }
                    1 => { let _ = board.reveal(cursor.0, cursor.1); }
                    2 => { let _ = board.toggle_flag((board.width()/2).min(board.width()-1), (board.height()/2).min(board.height()-1)); }
                    3 => { /* pause frame */ }
                    _ => break Ok(()),
                }
                demo_step += 1;
            }
        }
    };

    // teardown via guard; just ensure cursor visible
    terminal.show_cursor()?;
    res
}

fn ui(f: &mut ratatui::Frame, board: &Board, cursor: (usize, usize)) -> Rect {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(f.size());

    // Header
    let status = if !board.alive() {
        "Boom! You hit a mine — q to quit, n to restart"
    } else if board.won() {
        "You won! q to quit, n to restart"
    } else {
        "Mouse: left=reveal, right=flag • Arrows/HJKL move • Enter/Space reveal • f flag • n new • q quit"
    };
    let header = Paragraph::new(status)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Minesweeper"));
    f.render_widget(header, root[0]);

    // Board area
    let area = centered_grid_area(root[1], board.width() as u16, board.height() as u16);
    // Draw the board and compute the inner area used by cells (inside borders)
    let inner = inner_area(area);
    draw_board(f, board, area, cursor);

    let footer = Paragraph::new(format!("Size: {}x{}  Mines: {}", board.width(), board.height(), board.mines()))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, root[2]);
    inner
}

fn centered_grid_area(parent: Rect, cols: u16, rows: u16) -> Rect {
    let cell_w = 2; // one char + one space
    let cell_h = 1;
    let grid_w = cols * cell_w;
    let grid_h = rows * cell_h;
    let x = parent.x.saturating_add((parent.width.saturating_sub(grid_w)) / 2);
    let y = parent.y.saturating_add((parent.height.saturating_sub(grid_h)) / 2);
    Rect { x, y, width: grid_w.min(parent.width), height: grid_h.min(parent.height) }
}

fn draw_board(f: &mut ratatui::Frame, board: &Board, area: Rect, cursor: (usize, usize)) {
    // Build lines of text representing each row.
    let mut lines: Vec<Line> = Vec::with_capacity(board.height());
    for y in 0..board.height() {
        let mut spans: Vec<Span> = Vec::with_capacity(board.width() * 2);
        for x in 0..board.width() {
            let c = board.cell(x, y).unwrap();

            let mut ch = if !board.alive() && c.is_mine() { '*' } else if c.revealed() {
                if c.is_mine() { '*' } else if c.adjacent() == 0 { ' ' } else { char::from_digit(c.adjacent() as u32, 10).unwrap_or('?') }
            } else if c.flagged() { 'F' } else { '·' };

            // Color by state
            let mut style = if !board.alive() && c.is_mine() { Style::default().fg(Color::Red) }
                else if c.flagged() { Style::default().fg(Color::Yellow) }
                else if c.revealed() { number_style(c.adjacent()) } else { Style::default().fg(Color::DarkGray) };

            // Highlight selected cell
            if cursor.0 == x && cursor.1 == y {
                style = style.add_modifier(Modifier::REVERSED);
                if ch == ' ' { ch = '·'; }
            }

            spans.push(Span::styled(format!("{} ", ch), style));
        }
        lines.push(Line::from(spans));
    }

    let board_block = Block::default().borders(Borders::ALL).title("Board");
    let para = Paragraph::new(lines).block(board_block);
    f.render_widget(para, area);
}

fn number_style(n: u8) -> Style {
    match n {
        0 => Style::default().fg(Color::Gray),
        1 => Style::default().fg(Color::Blue),
        2 => Style::default().fg(Color::Green),
        3 => Style::default().fg(Color::Red),
        4 => Style::default().fg(Color::Magenta),
        5 => Style::default().fg(Color::Yellow),
        6 => Style::default().fg(Color::Cyan),
        _ => Style::default().fg(Color::White),
    }
}

fn inner_area(area: Rect) -> Rect {
    // Match Block::inner() for Borders::ALL: shrink by 1 on each side
    Rect { x: area.x.saturating_add(1), y: area.y.saturating_add(1), width: area.width.saturating_sub(2), height: area.height.saturating_sub(2) }
}

fn pos_to_cell(mx: u16, my: u16, inner: Rect, cols: u16, rows: u16) -> Option<(u16, u16)> {
    if mx < inner.x || my < inner.y { return None; }
    let rel_x = mx - inner.x;
    let rel_y = my - inner.y;
    let cell_w = 2u16; // must match centered_grid_area and rendering width
    let cx = rel_x / cell_w;
    let cy = rel_y / 1u16;
    if cx < cols && cy < rows { Some((cx, cy)) } else { None }
}

struct TermGuard;
impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        use crossterm::ExecutableCommand;
        let mut stdout = std::io::stdout();
        let _ = stdout.execute(DisableMouseCapture);
        let _ = stdout.execute(LeaveAlternateScreen);
    }
}
