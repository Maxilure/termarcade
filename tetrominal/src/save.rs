use std::fs;
use std::path::Path;

use crate::board::{Board, BOARD_W, TOTAL_H};
use crate::controls::ControlScheme;
use crate::game::GameMode;
use crate::options::Options;
use crate::score::ScoreManager;
use crate::tetromino::PieceType;

const SAVE_PATH: &str = "tetrominal.save";

pub fn save_exists() -> bool {
    Path::new(SAVE_PATH).exists()
}

pub fn delete_save() {
    let _ = fs::remove_file(SAVE_PATH);
}

#[derive(Clone)]
pub struct SaveData {
    pub board: Board,
    pub score: ScoreManager,
    pub current_piece: PieceType,
    pub current_x: i16,
    pub current_y: i16,
    pub current_rot: usize,
    pub next_piece: PieceType,
    pub hold_piece: Option<PieceType>,
    pub bag: Vec<PieceType>,
    pub can_hold: bool,
    pub tick_accum: f64,
    pub lock_timer: f64,
    pub lock_resets: u32,
    pub game_mode: GameMode,
}

fn encode_piece(p: Option<PieceType>) -> u8 {
    match p {
        None => 0xFF,
        Some(PieceType::I) => 0,
        Some(PieceType::O) => 1,
        Some(PieceType::T) => 2,
        Some(PieceType::S) => 3,
        Some(PieceType::Z) => 4,
        Some(PieceType::J) => 5,
        Some(PieceType::L) => 6,
    }
}

fn decode_piece(b: u8) -> Option<PieceType> {
    match b {
        0 => Some(PieceType::I),
        1 => Some(PieceType::O),
        2 => Some(PieceType::T),
        3 => Some(PieceType::S),
        4 => Some(PieceType::Z),
        5 => Some(PieceType::J),
        6 => Some(PieceType::L),
        _ => None,
    }
}

pub fn save_game(data: &SaveData) -> std::io::Result<()> {
    let mut buf = Vec::new();

    buf.extend_from_slice(b"TETRSAVE");

    for row in &data.board.cells {
        for &cell in row {
            buf.push(encode_piece(cell));
        }
    }

    buf.extend_from_slice(&data.score.score.to_le_bytes());
    buf.extend_from_slice(&data.score.level.to_le_bytes());
    buf.extend_from_slice(&data.score.lines.to_le_bytes());

    buf.push(data.current_piece as u8);
    buf.push(data.current_x as i8 as u8);
    buf.push(data.current_y as i8 as u8);
    buf.push(data.current_rot as u8);

    buf.push(data.next_piece as u8);
    buf.push(encode_piece(data.hold_piece));
    buf.push(data.can_hold as u8);

    let bag_len = data.bag.len() as u8;
    buf.push(bag_len);
    for &p in &data.bag {
        buf.push(p as u8);
    }

    buf.extend_from_slice(&data.tick_accum.to_le_bytes());
    buf.extend_from_slice(&data.lock_timer.to_le_bytes());
    buf.extend_from_slice(&data.lock_resets.to_le_bytes());

    buf.push(match data.game_mode {
        GameMode::Classic => 0,
        GameMode::Relaxed => 1,
    });

    fs::write(SAVE_PATH, &buf)
}

pub fn load_game() -> std::io::Result<SaveData> {
    let buf = fs::read(SAVE_PATH)?;

    if buf.len() < 30 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "save file too short"));
    }

    if &buf[0..8] != b"TETRSAVE" {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad save magic"));
    }

    let mut off = 8;

    let mut board = Board::new();
    for row in 0..TOTAL_H {
        for col in 0..BOARD_W {
            if off >= buf.len() {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "truncated board data"));
            }
            board.cells[row][col] = decode_piece(buf[off]);
            off += 1;
        }
    }

    let read_u32 = |off: &mut usize| -> std::io::Result<u32> {
        if *off + 4 > buf.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "truncated u32"));
        }
        let val = u32::from_le_bytes(buf[*off..*off + 4].try_into().unwrap());
        *off += 4;
        Ok(val)
    };

    let read_f64 = |off: &mut usize| -> std::io::Result<f64> {
        if *off + 8 > buf.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "truncated f64"));
        }
        let val = f64::from_le_bytes(buf[*off..*off + 8].try_into().unwrap());
        *off += 8;
        Ok(val)
    };

    let score_val = read_u32(&mut off)?;
    let level_val = read_u32(&mut off)?;
    let lines_val = read_u32(&mut off)?;

    let mut score = ScoreManager::new();
    score.score = score_val;
    score.level = level_val;
    score.lines = lines_val;

    if off + 4 > buf.len() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "truncated current piece"));
    }
    let current_piece = match decode_piece(buf[off]) {
        Some(p) => p,
        None => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad current piece")),
    };
    off += 1;
    let current_x = buf[off] as i8 as i16;
    off += 1;
    let current_y = buf[off] as i8 as i16;
    off += 1;
    let current_rot = buf[off] as usize;
    off += 1;

    if off + 1 > buf.len() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "truncated next piece"));
    }
    let next_piece = match decode_piece(buf[off]) {
        Some(p) => p,
        None => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad next piece")),
    };
    off += 1;
    let hold_piece = decode_piece(buf[off]);
    off += 1;
    let can_hold = buf[off] != 0;
    off += 1;

    if off + 1 > buf.len() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "truncated bag len"));
    }
    let bag_len = buf[off] as usize;
    off += 1;

    let mut bag = Vec::with_capacity(bag_len);
    for _ in 0..bag_len {
        if off >= buf.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "truncated bag"));
        }
        bag.push(match decode_piece(buf[off]) {
            Some(p) => p,
            None => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad bag piece")),
        });
        off += 1;
    }

    let tick_accum = read_f64(&mut off)?;
    let lock_timer = read_f64(&mut off)?;
    let lock_resets = read_u32(&mut off)?;

    let game_mode = if off < buf.len() {
        match buf[off] {
            0 => GameMode::Classic,
            1 => GameMode::Relaxed,
            _ => GameMode::Classic,
        }
    } else {
        GameMode::Classic
    };

    Ok(SaveData {
        board,
        score,
        current_piece,
        current_x,
        current_y,
        current_rot,
        next_piece,
        hold_piece,
        bag,
        can_hold,
        tick_accum,
        lock_timer,
        lock_resets,
        game_mode,
    })
}

const PREF_PATH: &str = "tetrominal.pref";

pub fn save_pref(scheme: ControlScheme, options: &Options, mode: GameMode, classic_high: u32, relaxed_high: u32) {
    let mut buf = vec![scheme as u8, 0u8];
    if options.gray_blocks { buf[1] |= 1; }
    if options.ghost_piece { buf[1] |= 2; }
    if options.grid_lines { buf[1] |= 4; }
    if options.flash_on_clear { buf[1] |= 8; }
    if options.hard_drop_trail { buf[1] |= 16; }
    buf.push(match mode {
        GameMode::Classic => 0,
        GameMode::Relaxed => 1,
    });
    buf.extend_from_slice(&classic_high.to_le_bytes());
    buf.extend_from_slice(&relaxed_high.to_le_bytes());
    let _ = fs::write(PREF_PATH, &buf);
}

pub fn load_pref() -> Option<(ControlScheme, Options, GameMode, u32, u32)> {
    let buf = fs::read(PREF_PATH).ok()?;
    let (scheme, options, mode, classic_high, relaxed_high) = if buf.len() == 6 {
        let scheme = match buf[0] {
            0 => ControlScheme::Arrows,
            1 => ControlScheme::Wasd,
            2 => ControlScheme::Hjkl,
            _ => {
                let _ = fs::remove_file(PREF_PATH);
                return None;
            }
        };
        let options = Options {
            gray_blocks: buf[1] & 1 != 0,
            ghost_piece: buf[1] & 2 != 0,
            grid_lines: buf[1] & 4 != 0,
            flash_on_clear: buf[1] & 8 != 0,
            hard_drop_trail: buf[1] & 16 != 0,
        };
        let high_score = u32::from_le_bytes(buf[2..6].try_into().unwrap());
        (scheme, options, GameMode::Classic, high_score, 0)
    } else if buf.len() == 11 {
        let scheme = match buf[0] {
            0 => ControlScheme::Arrows,
            1 => ControlScheme::Wasd,
            2 => ControlScheme::Hjkl,
            _ => {
                let _ = fs::remove_file(PREF_PATH);
                return None;
            }
        };
        let options = Options {
            gray_blocks: buf[1] & 1 != 0,
            ghost_piece: buf[1] & 2 != 0,
            grid_lines: buf[1] & 4 != 0,
            flash_on_clear: buf[1] & 8 != 0,
            hard_drop_trail: buf[1] & 16 != 0,
        };
        let mode = match buf[2] {
            0 => GameMode::Classic,
            1 => GameMode::Relaxed,
            _ => GameMode::Classic,
        };
        let classic_high = u32::from_le_bytes(buf[3..7].try_into().unwrap());
        let relaxed_high = u32::from_le_bytes(buf[7..11].try_into().unwrap());
        (scheme, options, mode, classic_high, relaxed_high)
    } else {
        let _ = fs::remove_file(PREF_PATH);
        return None;
    };
    Some((scheme, options, mode, classic_high, relaxed_high))
}

