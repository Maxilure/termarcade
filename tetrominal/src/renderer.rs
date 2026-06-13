use crossterm::style::Color;
use termarcade_core::frame::{Cell, Frame, Style};

use crate::board::{Board, BOARD_H, BOARD_W, TOTAL_H};
use crate::controls::ControlScheme;
use crate::game::GameMode;
use crate::options::Options;
use crate::particles::ParticleSystem;
use crate::score::ScoreManager;
use crate::tetromino::PieceType;

#[derive(Clone, Copy, PartialEq)]
pub enum Overlay {
    None,
    Paused(usize),
    Options(usize),
    Controls(ControlScheme),
    GameOver(usize),
    Menu { cursor: usize, continue_avail: bool, saved_mode: GameMode },
    ModeSelect(usize),
}

const MIN_W: u16 = 42;
const MIN_H: u16 = 24;

const BUF_START: i16 = (TOTAL_H - BOARD_H) as i16;

const BORDER_TL: char = '┌';
const BORDER_TR: char = '┐';
const BORDER_BL: char = '└';
const BORDER_BR: char = '┘';
const BORDER_H: char = '─';
const BORDER_V: char = '│';

fn cell_dims(fw: u16, fh: u16) -> (u16, u16) {
    let from_w = (fw.saturating_sub(22)) / BOARD_W as u16;
    let from_h = if fh >= 24 { 2 * (fh - 4) / BOARD_H as u16 } else { 2 };
    let cell_w = ((from_w.min(from_h) / 2) * 2).clamp(2, 6);
    (cell_w, cell_w / 2)
}

pub fn render(
    frame: &mut Frame,
    board: &Board,
    score: &ScoreManager,
    overlay: Overlay,
    active: Option<(PieceType, i16, i16, usize)>,
    ghost_row: Option<i16>,
    next_piece: PieceType,
    hold_piece: Option<PieceType>,
    options: &Options,
    particles: &ParticleSystem,
    game_mode: GameMode,
    classic_high: u32,
    relaxed_high: u32,
    clear_display_timer: f64,
    clear_rows: &[usize],
    clear_origin_col: usize,
    clear_piece_cells: &[(usize, usize)],
) {
    frame.clear();

    let fw = frame.width();
    let fh = frame.height();

    if fw < MIN_W || fh < MIN_H {
        let msg = "Terminal too small (min 42x24)";
        let x = (fw.saturating_sub(msg.len() as u16)) / 2;
        let y = fh / 2;
        frame.text(x, y, msg, Style::new(Color::Red, Color::Reset));
        return;
    }

    let (cell_w, cell_h) = cell_dims(fw, fh);

    let board_chars_w: u16 = BOARD_W as u16 * cell_w;
    let board_total_w: u16 = board_chars_w + 2;
    let board_total_h: u16 = BOARD_H as u16 * cell_h + 2;

    let preview_cw: u16 = 2;
    let preview_ch: u16 = 1;
    let side_inner_w = (4 * preview_cw).max(10);
    let side_w = side_inner_w + 4;

    let content_w = board_total_w + 2 + side_w;
    let ox = (fw.saturating_sub(content_w)) / 2;
    let oy = ((fh as i16 - board_total_h as i16) / 2).max(1) as u16;

    let bx = ox;
    let by = oy;
    let sx = ox + board_total_w + 2;
    let sy = by;

    let fullscreen = matches!(overlay, Overlay::Menu { .. } | Overlay::GameOver(_) | Overlay::ModeSelect(_) | Overlay::Controls(_));

    if !fullscreen {
        draw_board_frame(frame, bx, by, cell_w, cell_h, clear_display_timer);

        if options.grid_lines {
            draw_grid_lines(frame, bx + 1, by + 1, cell_w, cell_h);
        }

        if clear_display_timer > 0.0 {
            draw_cells(frame, bx + 1, by + 1, board, cell_w, cell_h, options, clear_display_timer, clear_rows, clear_origin_col, clear_piece_cells);
            particles.render(frame, bx + 1, by + 1, cell_w, cell_h);
        } else {
            particles.render(frame, bx + 1, by + 1, cell_w, cell_h);
            draw_cells(frame, bx + 1, by + 1, board, cell_w, cell_h, options, clear_display_timer, clear_rows, clear_origin_col, clear_piece_cells);
        }

        if let Some((piece, px, py, prot)) = active {
            if let Some(gy) = ghost_row {
                if options.ghost_piece {
                    draw_piece(frame, bx + 1, by + 1, piece, px, gy, prot, true, cell_w, cell_h, options);
                }
            }
            draw_piece(frame, bx + 1, by + 1, piece, px, py, prot, false, cell_w, cell_h, options);
        }

        draw_side_panel(frame, sx, sy, side_inner_w, next_piece, hold_piece, score, game_mode, classic_high, relaxed_high, preview_cw, preview_ch, options);
    }

    match overlay {
        Overlay::None => {}
        Overlay::Paused(cursor) => draw_pause_overlay(frame, fw, fh, cursor),
        Overlay::Options(cursor) => draw_options_overlay(frame, fw, fh, cursor, options),
        Overlay::Controls(scheme) => draw_controls_overlay(frame, fw, fh, scheme),
        Overlay::GameOver(cursor) => draw_gameover_overlay(frame, fw, fh, score.score, cursor),
        Overlay::Menu { cursor, continue_avail, saved_mode } => draw_menu_overlay(frame, fw, fh, cursor, continue_avail, saved_mode, classic_high, relaxed_high),
        Overlay::ModeSelect(cursor) => draw_mode_select_overlay(frame, fw, fh, cursor),
    }
}

fn cell(c: char, fg: Color, bg: Color) -> Cell {
    Cell::new(c, Style::new(fg, bg))
}

fn sp(bg: Color) -> Cell {
    Cell::new(' ', Style::new(Color::Reset, bg))
}

fn border_rainbow(t: f64) -> Color {
    const COLORS: [Color; 6] = [Color::Red, Color::Yellow, Color::Green, Color::Cyan, Color::Blue, Color::Magenta];
    let idx = ((t * 50.0) as usize) % COLORS.len();
    COLORS[idx]
}

fn draw_board_frame(frame: &mut Frame, x: u16, y: u16, cell_w: u16, cell_h: u16, clear_timer: f64) {
    let w = BOARD_W as u16 * cell_w + 2;
    let h = BOARD_H as u16 * cell_h + 2;
    let bc = if clear_timer > 0.0 { border_rainbow(clear_timer) } else { Color::Reset };

    frame.set(x, y, cell(BORDER_TL, bc, Color::Reset));
    for cx in x + 1..x + w - 1 {
        frame.set(cx, y, cell(BORDER_H, bc, Color::Reset));
    }
    frame.set(x + w - 1, y, cell(BORDER_TR, bc, Color::Reset));

    for ry in y + 1..y + h - 1 {
        frame.set(x, ry, cell(BORDER_V, bc, Color::Reset));
        frame.set(x + w - 1, ry, cell(BORDER_V, bc, Color::Reset));
    }

    frame.set(x, y + h - 1, cell(BORDER_BL, bc, Color::Reset));
    for cx in x + 1..x + w - 1 {
        frame.set(cx, y + h - 1, cell(BORDER_H, bc, Color::Reset));
    }
    frame.set(x + w - 1, y + h - 1, cell(BORDER_BR, bc, Color::Reset));
}

fn draw_cells(
    frame: &mut Frame, x: u16, y: u16, board: &Board,
    cell_w: u16, cell_h: u16, options: &Options,
    clear_timer: f64, clear_rows: &[usize], clear_origin: usize,
    piece_cells: &[(usize, usize)],
) {
    let duration = 0.6;
    for row in 0..BOARD_H {
        let board_row = BUF_START as usize + row;
        let py = y + row as u16 * cell_h;
        let is_clearing = clear_timer > 0.0 && clear_rows.contains(&board_row);

        for col in 0..BOARD_W {
            let px = x + col as u16 * cell_w;
            if let Some(pt) = board.cells[board_row][col] {
                if is_clearing {
                    let progress = 1.0 - (clear_timer / duration);
                    let max_dist = clear_origin.max(BOARD_W - 1 - clear_origin) as f64;
                    let d = (col as i16 - clear_origin as i16).unsigned_abs() as f64;
                    let is_piece = piece_cells.contains(&(col, board_row));

                    let phase1_end = 0.2;
                    let vanished = if is_piece {
                        progress >= phase1_end
                    } else if progress >= phase1_end {
                        let wave_p = (progress - phase1_end) / (1.0 - phase1_end);
                        let front = wave_p * max_dist;
                        d <= front
                    } else {
                        false
                    };

                    if !vanished {
                        let bg = if options.gray_blocks { Color::DarkGrey } else { pt.color() };
                        for dy in 0..cell_h {
                            for dx in 0..cell_w {
                                frame.set(px + dx, py + dy, sp(bg));
                            }
                        }
                    }
                } else {
                    let bg = if options.gray_blocks { Color::DarkGrey } else { pt.color() };
                    for dy in 0..cell_h {
                        for dx in 0..cell_w {
                            frame.set(px + dx, py + dy, sp(bg));
                        }
                    }
                }
            }
        }
    }
}

fn draw_grid_lines(frame: &mut Frame, x: u16, y: u16, cell_w: u16, cell_h: u16) {
    for row in (4..BOARD_H).step_by(5) {
        let ly = y + row as u16 * cell_h - 1;
        for col in 0..BOARD_W {
            let px = x + col as u16 * cell_w;
            for dx in 0..cell_w {
                frame.set(px + dx, ly, cell('─', Color::DarkGrey, Color::Reset));
            }
        }
    }
}

fn draw_piece(
    frame: &mut Frame,
    ox: u16,
    oy: u16,
    piece: PieceType,
    px: i16,
    py: i16,
    rot: usize,
    ghost: bool,
    cell_w: u16,
    cell_h: u16,
    options: &Options,
) {
    let shape = piece.shape(rot);
    for (ri, &mask) in shape.iter().enumerate() {
        if mask == 0 {
            continue;
        }
        let vis_y = py + ri as i16 - BUF_START;
        if vis_y < 0 || vis_y >= BOARD_H as i16 {
            continue;
        }
        for col in 0..16u16 {
            if mask & (1 << col) != 0 {
                let fx = ox + (px + col as i16) as u16 * cell_w;
                let fy0 = oy + vis_y as u16 * cell_h;
                if ghost {
                    if options.gray_blocks {
                        let gs = Style::new(Color::DarkGrey, Color::Reset);
                        for dy in 0..cell_h {
                            for dx in 0..cell_w {
                                frame.set(fx + dx, fy0 + dy, style_cell('\u{2591}', gs));
                            }
                        }
                    } else {
                        let gs = Style::new(Color::DarkGrey, Color::DarkGrey);
                        for dy in 0..cell_h {
                            for dx in 0..cell_w {
                                frame.set(fx + dx, fy0 + dy, style_cell(' ', gs));
                            }
                        }
                    }
                } else {
                    let bg = if options.gray_blocks { Color::DarkGrey } else { piece.color() };
                    for dy in 0..cell_h {
                        for dx in 0..cell_w {
                            frame.set(fx + dx, fy0 + dy, sp(bg));
                        }
                    }
                }
            }
        }
    }
}

fn style_cell(c: char, style: Style) -> Cell {
    Cell::new(c, style)
}

fn draw_side_panel(
    frame: &mut Frame,
    x: u16, y: u16,
    inner_w: u16,
    next_piece: PieceType,
    hold_piece: Option<PieceType>,
    score: &ScoreManager,
    game_mode: GameMode,
    classic_high: u32,
    relaxed_high: u32,
    pw: u16, ph: u16,
    options: &Options,
) {
    let box_w = inner_w + 4;
    let inner_x = x + 2;
    let preview_ox = inner_x + (inner_w - 4 * pw) / 2;
    let label = Style::new(Color::White, Color::Reset);
    let iw = inner_w as usize;
    let high_score = match game_mode {
        GameMode::Classic => classic_high,
        GameMode::Relaxed => relaxed_high,
    };

    let mut cur_y = y;

    draw_box(frame, x, cur_y, box_w, 9);
    frame.text(inner_x, cur_y + 1, "NEXT", label);
    draw_scaled_preview(frame, preview_ox, cur_y + 2, Some(next_piece), pw, ph, options);
    for cx in inner_x..inner_x + inner_w {
        frame.set(cx, cur_y + 4, cell(BORDER_H, Color::DarkGrey, Color::Reset));
    }
    frame.text(inner_x, cur_y + 5, "HOLD", label);
    draw_scaled_preview(frame, preview_ox, cur_y + 6, hold_piece, pw, ph, options);
    cur_y += 10;

    draw_box(frame, x, cur_y, box_w, 7);
    frame.text(inner_x, cur_y + 1, &format!("{:<w$}", "SCORE", w = iw), label);
    frame.text(inner_x, cur_y + 2, &format!("{:>w$}", format!("{:08}", score.score), w = iw), Style::new(Color::Yellow, Color::Reset));
    if game_mode == GameMode::Classic {
        draw_stat(frame, inner_x, cur_y + 3, "LEVEL ", score.level, iw, Color::Cyan);
    }
    draw_stat(frame, inner_x, cur_y + 4, "LINES ", score.lines, iw, Color::Green);
    draw_stat(frame, inner_x, cur_y + 5, "HIGH ", high_score, iw, Color::Magenta);
}

fn draw_box(frame: &mut Frame, x: u16, y: u16, w: u16, h: u16) {
    frame.set(x, y, cell(BORDER_TL, Color::Reset, Color::Reset));
    for cx in x + 1..x + w - 1 {
        frame.set(cx, y, cell(BORDER_H, Color::Reset, Color::Reset));
    }
    frame.set(x + w - 1, y, cell(BORDER_TR, Color::Reset, Color::Reset));
    for ry in y + 1..y + h - 1 {
        frame.set(x, ry, cell(BORDER_V, Color::Reset, Color::Reset));
        frame.set(x + w - 1, ry, cell(BORDER_V, Color::Reset, Color::Reset));
    }
    frame.set(x, y + h - 1, cell(BORDER_BL, Color::Reset, Color::Reset));
    for cx in x + 1..x + w - 1 {
        frame.set(cx, y + h - 1, cell(BORDER_H, Color::Reset, Color::Reset));
    }
    frame.set(x + w - 1, y + h - 1, cell(BORDER_BR, Color::Reset, Color::Reset));
}

fn draw_stat(frame: &mut Frame, x: u16, y: u16, prefix: &str, val: u32, width: usize, fg: Color) {
    let num = format!("{}", val);
    let pad = width.saturating_sub(prefix.len() + num.len());
    let s = format!("{}{}{}", prefix, " ".repeat(pad), num);
    frame.text(x, y, &s, Style::new(fg, Color::Reset));
}

fn draw_scaled_preview(
    frame: &mut Frame,
    x: u16, y: u16,
    piece: Option<PieceType>,
    pw: u16, ph: u16,
    options: &Options,
) {
    let Some(piece) = piece else { return };

    let shape = piece.shape(0);
    let bw_box = if piece == PieceType::I || piece == PieceType::O {
        4u16
    } else {
        3u16
    };

    for (ri, &mask) in shape.iter().enumerate() {
        if mask == 0 { continue; }
        if ri >= 4 { break; }
        for col in 0..16u16 {
            if mask & (1 << col) != 0 {
                if col >= bw_box { break; }
                let c = if options.gray_blocks { Color::DarkGrey } else { piece.color() };
                let px = x + col * pw;
                let py = y + ri as u16 * ph;
                for dy in 0..ph {
                    for dx in 0..pw {
                        frame.set(px + dx, py + dy, cell('█', c, Color::Reset));
                    }
                }
            }
        }
    }
}

fn draw_pause_overlay(frame: &mut Frame, fw: u16, fh: u16, cursor: usize) {
    let lines = [
        "  ┌─────────────────┐  ",
        "  │     PAUSED      │  ",
        "  │                 │  ",
        "  │    Resume       │  ",
        "  │    Restart      │  ",
        "  │    Options      │  ",
        "  │    Main Menu    │  ",
        "  │    Exit         │  ",
        "  │                 │  ",
        "  │  P - Unpause    │  ",
        "  └─────────────────┘  ",
    ];
    let w = 21u16;
    let h = lines.len() as u16;
    let ox = (fw - w) / 2;
    let oy = (fh - h) / 2;
    for (i, line) in lines.iter().enumerate() {
        let opt_line = 3 + cursor as u16;
        let style = if i as u16 == opt_line {
            Style::new(Color::White, Color::Reset)
        } else {
            Style::new(Color::Yellow, Color::Reset)
        };
        frame.text(ox, oy + i as u16, line, style);
    }
    let cursor_y = oy + 3 + cursor as u16;
    frame.set(ox + 4, cursor_y, Cell::new('→', Style::new(Color::White, Color::Reset)));
}

fn draw_options_overlay(frame: &mut Frame, fw: u16, fh: u16, cursor: usize, options: &Options) {
    let n = Options::count();
    let bw: u16 = 32;
    let inner_w = bw - 2;

    let title_line = format!("  │{:^inner_w$}│  ", "OPTIONS", inner_w = inner_w as usize);
    let empty_line = format!("  │{:^inner_w$}│  ", "", inner_w = inner_w as usize);

    let toggle_line = format!("  │  {:<inner_w$}│  ", "Enter - Toggle", inner_w = (inner_w - 2) as usize);
    let back_line = format!("  │  {:<inner_w$}│  ", "Esc - Back", inner_w = (inner_w - 2) as usize);

    let border_top = format!("  ┌{}┐  ", "─".repeat(inner_w as usize));
    let border_bot = format!("  └{}┘  ", "─".repeat(inner_w as usize));

    let mut lines: Vec<String> = Vec::new();
    lines.push(border_top);
    lines.push(title_line);
    lines.push(empty_line.clone());
    for i in 0..n {
        let arrow = if i == cursor { "→" } else { " " };
        let content = format!("{}  {:<14}  {:<3}", arrow, Options::label(i), options.val(i));
        lines.push(format!("  │  {:<inner_w$}│  ", content, inner_w = (inner_w - 2) as usize));
    }
    {
        let arrow = if n == cursor { "→" } else { " " };
        let content = format!("{}  {:<14}", arrow, "Controls");
        lines.push(format!("  │  {:<inner_w$}│  ", content, inner_w = (inner_w - 2) as usize));
    }
    lines.push(empty_line);
    lines.push(toggle_line);
    lines.push(back_line);
    lines.push(border_bot);

    let w = bw + 4;
    let h = lines.len() as u16;
    let ox = (fw.saturating_sub(w)) / 2;
    let oy = (fh.saturating_sub(h)) / 2;
    let opt_start: u16 = 3;
    for (i, line) in lines.iter().enumerate() {
        let i = i as u16;
        let opt_line = opt_start + cursor as u16;
        let style = if i == opt_line {
            Style::new(Color::White, Color::Reset)
        } else {
            Style::new(Color::Cyan, Color::Reset)
        };
        frame.text(ox, oy + i, line, style);
    }
}

fn draw_controls_overlay(frame: &mut Frame, fw: u16, fh: u16, scheme: ControlScheme) {
    let sname = scheme.name();
    let bindings = scheme.bindings();
    let bw: u16 = 32;
    let inner_w = bw - 2;

    let border_top = format!("  ┌{}┐  ", "─".repeat(inner_w as usize));
    let border_bot = format!("  └{}┘  ", "─".repeat(inner_w as usize));
    let title_line = format!("  │{:^inner_w$}│  ", "CONTROLS", inner_w = inner_w as usize);
    let empty_line = format!("  │{:^inner_w$}│  ", "", inner_w = inner_w as usize);

    let scheme_line = format!("  │  Scheme: {:<inner_w$}│  ", sname, inner_w = (inner_w - 10) as usize);

    let change_line = format!("  │  {:<inner_w$}│  ", "\u{2190}\u{2192} change scheme", inner_w = (inner_w - 2) as usize);
    let back_line = format!("  │  {:<inner_w$}│  ", "Any key to go back", inner_w = (inner_w - 2) as usize);

    let mut lines: Vec<String> = Vec::new();
    lines.push(border_top);
    lines.push(title_line);
    lines.push(empty_line.clone());
    lines.push(scheme_line);
    lines.push(empty_line.clone());
    for (key_str, action) in &bindings {
        let content = format!("{:<9} {}", key_str, action);
        lines.push(format!("  │  {:<inner_w$}│  ", content, inner_w = (inner_w - 2) as usize));
    }
    lines.push(empty_line);
    lines.push(change_line);
    lines.push(back_line);
    lines.push(border_bot);

    let w = bw + 4;
    let h = lines.len() as u16;
    let ox = (fw.saturating_sub(w)) / 2;
    let oy = (fh.saturating_sub(h)) / 2;
    for (i, line) in lines.iter().enumerate() {
        frame.text(ox, oy + i as u16, line, Style::new(Color::Cyan, Color::Reset));
    }
}

fn draw_gameover_overlay(frame: &mut Frame, fw: u16, fh: u16, score_val: u32, cursor: usize) {
    let score_str = format!("{:08}", score_val);
    let bw: u16 = 28;
    let inner_w = bw - 2;
    let iw = inner_w as usize;

    let border_top = format!("  ┌{}┐  ", "─".repeat(iw));
    let border_bot = format!("  └{}┘  ", "─".repeat(iw));
    let empty_line = format!("  │{:^iw$}│  ", "", iw = iw);

    let options = ["Restart", "Return to Main Menu", "Quit"];

    let mut lines: Vec<String> = Vec::new();
    lines.push(border_top);
    lines.push(format!("  │{:^iw$}│  ", "GAME OVER", iw = iw));
    lines.push(empty_line.clone());
    lines.push(format!("  │  Score: {:<iw$}│  ", score_str, iw = iw - 9));
    lines.push(empty_line.clone());
    for opt in &options {
        lines.push(format!("  │    {:<iw$}│  ", opt, iw = iw - 4));
    }
    lines.push(border_bot);

    let w = bw + 4;
    let h = lines.len() as u16;
    let ox = (fw.saturating_sub(w)) / 2;
    let oy = (fh.saturating_sub(h)) / 2;

    let opt_start: u16 = 5;
    for (i, line) in lines.iter().enumerate() {
        let i = i as u16;
        let style = if i >= opt_start && i < opt_start + options.len() as u16 {
            if i == opt_start + cursor as u16 {
                Style::new(Color::White, Color::Reset)
            } else {
                Style::new(Color::Yellow, Color::Reset)
            }
        } else {
            Style::new(Color::Red, Color::Reset)
        };
        frame.text(ox, oy + i, line, style);
    }

    let cursor_y = oy + opt_start + cursor as u16;
    frame.set(ox + 5, cursor_y, Cell::new('→', Style::new(Color::White, Color::Reset)));
}

fn draw_menu_overlay(frame: &mut Frame, fw: u16, fh: u16, cursor: usize, continue_avail: bool, saved_mode: GameMode, classic_high: u32, relaxed_high: u32) {
    let saved_mode_name = match saved_mode {
        GameMode::Classic => "C",
        GameMode::Relaxed => "R",
    };
    let continue_line = format!("  ║    Continue ({})           ║  ", saved_mode_name);
    let lines: Vec<String> = if continue_avail {
        vec![
            "  ╔═══════════════════════════╗  ".to_string(),
            "  ║                           ║  ".to_string(),
            "  ║        TETROMINAL         ║  ".to_string(),
            "  ║                           ║  ".to_string(),
            continue_line,
            "  ║    New Game               ║  ".to_string(),
            "  ║    Options                ║  ".to_string(),
            "  ║    Quit                   ║  ".to_string(),
            "  ║                           ║  ".to_string(),
            "  ║  ─────────────────────    ║  ".to_string(),
            "  ║  HIGH SCORES              ║  ".to_string(),
            format!("  ║  Classic {:>15}  ║  ", classic_high),
            format!("  ║  Relaxed {:>15}  ║  ", relaxed_high),
            "  ║                           ║  ".to_string(),
            "  ╚═══════════════════════════╝  ".to_string(),
        ]
    } else {
        vec![
            "  ╔═══════════════════════════╗  ".to_string(),
            "  ║                           ║  ".to_string(),
            "  ║        TETROMINAL         ║  ".to_string(),
            "  ║                           ║  ".to_string(),
            "  ║    New Game               ║  ".to_string(),
            "  ║    Options                ║  ".to_string(),
            "  ║    Quit                   ║  ".to_string(),
            "  ║                           ║  ".to_string(),
            "  ║  ─────────────────────    ║  ".to_string(),
            "  ║  HIGH SCORES              ║  ".to_string(),
            format!("  ║  Classic {:>15}  ║  ", classic_high),
            format!("  ║  Relaxed {:>15}  ║  ", relaxed_high),
            "  ║                           ║  ".to_string(),
            "  ╚═══════════════════════════╝  ".to_string(),
        ]
    };

    let w = 33u16;
    let h = lines.len() as u16;
    let ox = (fw.saturating_sub(w)) / 2;
    let oy = (fh.saturating_sub(h)) / 2;

    let opt_start: u16 = 4;
    let opt_count = if continue_avail { 4 } else { 3 };
    for (i, line) in lines.iter().enumerate() {
        let i = i as u16;
        let style = if i >= opt_start && i < opt_start + opt_count {
            if i == opt_start + cursor as u16 {
                Style::new(Color::White, Color::Reset)
            } else {
                Style::new(Color::Green, Color::Reset)
            }
        } else if i >= 10 && i <= 12 {
            Style::new(Color::Cyan, Color::Reset)
        } else {
            Style::new(Color::Green, Color::Reset)
        };
        frame.text(ox, oy + i, line, style);
    }

    let cursor_y = oy + opt_start + cursor as u16;
    frame.set(ox + 4, cursor_y, Cell::new('→', Style::new(Color::White, Color::Reset)));
}

fn draw_mode_select_overlay(frame: &mut Frame, fw: u16, fh: u16, cursor: usize) {
    let lines = [
        "  ┌─────────────────┐  ",
        "  │   SELECT MODE   │  ",
        "  │                 │  ",
        "  │    Classic      │  ",
        "  │    Relaxed      │  ",
        "  │                 │  ",
        "  │    Back         │  ",
        "  └─────────────────┘  ",
    ];
    let w = 21u16;
    let h = lines.len() as u16;
    let ox = (fw - w) / 2;
    let oy = (fh - h) / 2;
    let opt_start: u16 = 3;
    let opt_gap: u16 = 1;
    for (i, line) in lines.iter().enumerate() {
        let i = i as u16;
        let opt_line = opt_start + cursor as u16 + if cursor as u16 >= 2 { opt_gap } else { 0 };
        let style = if i == opt_line {
            Style::new(Color::White, Color::Reset)
        } else {
            Style::new(Color::Green, Color::Reset)
        };
        frame.text(ox, oy + i, line, style);
    }
    let cursor_y = oy + opt_start + cursor as u16 + if cursor as u16 >= 2 { opt_gap } else { 0 };
    frame.set(ox + 4, cursor_y, Cell::new('→', Style::new(Color::White, Color::Reset)));
}
