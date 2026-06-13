use std::time::Duration;

use rand::seq::SliceRandom;
use termarcade_core::frame::Frame;
use termarcade_core::game::Game;
use termarcade_core::input::Key;

use crate::board::{Board, BOARD_W, TOTAL_H};
use crate::controls::{key_to_action, ControlScheme};
use crate::options::Options;
use crate::particles::ParticleSystem;
use crate::save;

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    Classic,
    Relaxed,
}

#[derive(Clone, Copy, PartialEq)]
enum PausedView {
    Menu,
    Options,
    Controls,
}
use crate::collision;
use crate::renderer::{self, Overlay};
use crate::score::ScoreManager;
use crate::tetromino::{PieceType, PIECE_TYPES};

const LOCK_DELAY: f64 = 0.5;
const MAX_LOCK_RESETS: u32 = 15;

#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    Menu,
    ModeSelect,
    Options,
    Playing,
    Paused,
    GameOver,
}

pub struct BlockDropGame {
    board: Board,
    score: ScoreManager,
    game_mode: GameMode,
    state: GameState,
    running: bool,

    current_piece: PieceType,
    current_x: i16,
    current_y: i16,
    current_rot: usize,

    next_piece: PieceType,
    hold_piece: Option<PieceType>,
    can_hold: bool,

    bag: Vec<PieceType>,
    tick_accum: f64,
    lock_timer: f64,
    lock_resets: u32,

    paused_view: PausedView,
    paused_cursor: usize,

    options: Options,
    control_scheme: ControlScheme,

    particles: ParticleSystem,

    clear_display_timer: f64,
    clear_rows: Vec<usize>,
    clear_origin_col: usize,
    clear_piece_cells: Vec<(usize, usize)>,
    clear_pending_cells: Vec<(usize, usize, PieceType)>,

    menu_cursor: usize,
    menu_has_continue: bool,
    saved_game_mode: GameMode,
    mode_select_cursor: usize,
    gameover_cursor: usize,
    viewing_controls: bool,
    classic_high: u32,
    relaxed_high: u32,
}

impl BlockDropGame {
    pub fn new() -> Self {
        let mut game = BlockDropGame {
            board: Board::new(),
            score: ScoreManager::new(),
            game_mode: GameMode::Classic,
            state: GameState::Menu,
            running: true,
            current_piece: PieceType::I,
            current_x: 3,
            current_y: 0,
            current_rot: 0,
            next_piece: PieceType::T,
            hold_piece: None,
            can_hold: true,
            bag: Vec::new(),
            tick_accum: 0.0,
            lock_timer: 0.0,
            lock_resets: 0,

            paused_view: PausedView::Menu,
            paused_cursor: 0,

            control_scheme: ControlScheme::Arrows,
            options: Options::default(),

            particles: ParticleSystem::new(),

            clear_display_timer: 0.0,
            clear_rows: Vec::new(),
            clear_origin_col: 0,
            clear_piece_cells: Vec::new(),
            clear_pending_cells: Vec::new(),

            menu_cursor: 0,
            menu_has_continue: save::save_exists(),
            saved_game_mode: GameMode::Classic,
            mode_select_cursor: 0,
            gameover_cursor: 0,
            viewing_controls: false,

            classic_high: 0,
            relaxed_high: 0,
        };
        game.fill_bag();
        game.next_piece = game.bag.remove(0);
        if let Some((scheme, opts, mode, ch, rh)) = save::load_pref() {
            game.control_scheme = scheme;
            game.options = opts;
            game.game_mode = mode;
            game.classic_high = ch;
            game.relaxed_high = rh;
        }
        if game.menu_has_continue {
            if let Ok(data) = save::load_game() {
                game.saved_game_mode = data.game_mode;
            }
        }
        game
    }

    fn fill_bag(&mut self) {
        let mut bag: Vec<PieceType> = PIECE_TYPES.to_vec();
        bag.shuffle(&mut rand::thread_rng());
        self.bag.extend(bag);
    }

    fn start_game(&mut self) {
        self.board.reset();
        self.score.reset();
        self.hold_piece = None;
        self.tick_accum = 0.0;
        self.lock_timer = 0.0;
        self.lock_resets = 0;
        self.particles.clear();
        self.clear_display_timer = 0.0;
        self.clear_rows.clear();
        self.clear_origin_col = 0;
        self.clear_piece_cells.clear();
        self.clear_pending_cells.clear();
        self.paused_view = PausedView::Menu;
        self.paused_cursor = 0;
        self.menu_cursor = 0;
        self.gameover_cursor = 0;
        save::delete_save();
        self.bag.clear();
        self.fill_bag();
        self.next_piece = self.bag.remove(0);
        self.spawn_from_bag();
        self.state = GameState::Playing;
    }

    fn spawn_from_bag(&mut self) {
        if self.bag.len() < 7 {
            self.fill_bag();
        }
        self.current_piece = self.next_piece;
        self.next_piece = self.bag.remove(0);
        self.current_rot = 0;
        self.current_x = self.current_piece.spawn_x();
        self.current_y = self.current_piece.spawn_y();
        self.tick_accum = 0.0;
        self.lock_timer = 0.0;
        self.lock_resets = 0;
        self.can_hold = true;

        if self
            .board
            .collides(self.current_piece, self.current_x, self.current_y, 0)
        {
            self.state = GameState::GameOver;
            if self.score.score > self.current_high_score() {
                self.set_high_score(self.score.score);
            }
        }
    }

    fn get_piece_cells(&self, piece: PieceType, x: i16, y: i16, rot: usize) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        let shape = piece.shape(rot);
        for (ri, &mask) in shape.iter().enumerate() {
            if mask == 0 {
                continue;
            }
            let row = y + ri as i16;
            if row < 0 || row >= TOTAL_H as i16 {
                continue;
            }
            for col in 0..16u16 {
                if mask & (1 << col) != 0 {
                    let cx = x + col as i16;
                    if cx >= 0 && cx < BOARD_W as i16 {
                        cells.push((cx as usize, row as usize));
                    }
                }
            }
        }
        cells
    }

    fn lock_and_spawn(&mut self) {
        self.board
            .lock(self.current_piece, self.current_x, self.current_y, self.current_rot);

        let full_rows = self.board.find_full_rows();
        let cleared = full_rows.len() as u32;

        if cleared > 0 {
            self.score.add_lines(cleared);

            if cleared >= 3 && self.options.flash_on_clear {
                let cells = self.board.capture_cleared_cells();
                let piece_cells = self.get_piece_cells(
                    self.current_piece,
                    self.current_x,
                    self.current_y,
                    self.current_rot,
                );
                let in_cleared: Vec<(usize, usize)> = piece_cells
                    .into_iter()
                    .filter(|(_, row)| full_rows.contains(row))
                    .collect();

                let origin = self.current_x
                    + match self.current_piece {
                        PieceType::I => 2,
                        _ => 1,
                    };

                self.clear_rows = full_rows;
                self.clear_origin_col = origin as usize;
                self.clear_piece_cells = in_cleared;
                self.clear_pending_cells = cells;
                self.clear_display_timer = 0.6;
            } else {
                let cleared_cells = if self.options.flash_on_clear {
                    self.board.capture_cleared_cells()
                } else {
                    Vec::new()
                };
                self.board.clear_lines_detailed();
                if self.options.flash_on_clear && !cleared_cells.is_empty() {
                    self.particles
                        .spawn_line_clear(&cleared_cells, self.options.gray_blocks, false);
                }
            }
        }

        self.spawn_from_bag();
    }

    fn try_move(&mut self, dx: i16) {
        if !self
            .board
            .collides(self.current_piece, self.current_x + dx, self.current_y, self.current_rot)
        {
            self.current_x += dx;
            if self
                .board
                .collides(self.current_piece, self.current_x, self.current_y + 1, self.current_rot)
            {
                self.reset_lock_delay();
            }
        }
    }

    fn try_rotate(&mut self, dir: i8) {
        let from = self.current_rot;
        let to = ((self.current_rot as i8 + dir).rem_euclid(4)) as usize;

        if let Some((nx, ny, nrot)) = collision::try_rotate(
            |x, y, r| self.board.collides(self.current_piece, x, y, r),
            self.current_piece,
            self.current_x,
            self.current_y,
            from,
            to,
        ) {
            self.current_x = nx;
            self.current_y = ny;
            self.current_rot = nrot;
            if self
                .board
                .collides(self.current_piece, self.current_x, self.current_y + 1, self.current_rot)
            {
                self.reset_lock_delay();
            }
        }
    }

    fn soft_drop(&mut self) {
        if !self
            .board
            .collides(self.current_piece, self.current_x, self.current_y + 1, self.current_rot)
        {
            self.current_y += 1;
            self.score.add_soft_drop();
            self.tick_accum = 0.0;
            self.lock_timer = 0.0;
            self.lock_resets = 0;
        }
    }

    fn hard_drop(&mut self) {
        let gy = self
            .board
            .ghost_y(self.current_piece, self.current_x, self.current_y, self.current_rot);
        let dist = (gy - self.current_y) as u32;
        if dist > 0 {
            self.score.add_hard_drop(dist);
            if self.options.hard_drop_trail {
                self.particles.spawn_hard_drop_trail(
                    self.current_piece,
                    self.current_rot,
                    self.current_x,
                    self.current_y,
                    gy,
                    self.options.gray_blocks,
                );
            }
            self.current_y = gy;
        }
        self.lock_and_spawn();
    }

    fn hold(&mut self) {
        if !self.can_hold {
            return;
        }
        self.can_hold = false;
        let cur = self.current_piece;
        match self.hold_piece {
            Some(held) => {
                self.hold_piece = Some(cur);
                self.current_piece = held;
                self.current_rot = 0;
                self.current_x = held.spawn_x();
                self.current_y = held.spawn_y();
                self.tick_accum = 0.0;
                self.lock_timer = 0.0;
                self.lock_resets = 0;
                if self
                    .board
                    .collides(self.current_piece, self.current_x, self.current_y, 0)
                {
                    self.state = GameState::GameOver;
                    if self.score.score > self.current_high_score() {
                        self.set_high_score(self.score.score);
                    }
                }
            }
            None => {
                self.hold_piece = Some(cur);
                self.spawn_from_bag();
            }
        }
    }

    fn reset_lock_delay(&mut self) {
        if self.lock_resets < MAX_LOCK_RESETS {
            self.lock_timer = 0.0;
            self.lock_resets += 1;
        }
    }

    fn current_high_score(&self) -> u32 {
        match self.game_mode {
            GameMode::Classic => self.classic_high,
            GameMode::Relaxed => self.relaxed_high,
        }
    }

    fn set_high_score(&mut self, score: u32) {
        match self.game_mode {
            GameMode::Classic => self.classic_high = score,
            GameMode::Relaxed => self.relaxed_high = score,
        }
    }

    pub fn with_mode(mode: GameMode) -> Self {
        let mut game = BlockDropGame::new();
        game.game_mode = mode;
        game.menu_has_continue = false;
        game.start_game();
        game
    }
}

impl Game for BlockDropGame {
    fn tick(&mut self, dt: Duration, input: &[Key]) {
        let dt = dt.as_secs_f64();

        match self.state {
            GameState::Menu => {
                let max = if self.menu_has_continue { 3 } else { 2 };
                for k in input {
                    match k {
                        Key::Up => {
                            self.menu_cursor = self.menu_cursor.saturating_sub(1);
                        }
                        Key::Down => {
                            self.menu_cursor = (self.menu_cursor + 1).min(max);
                        }
                        Key::Enter => {
                            let opt_idx = if self.menu_has_continue { self.menu_cursor } else { self.menu_cursor + 1 };
                            match opt_idx {
                                0 => {
                                    match save::load_game() {
                                        Ok(data) => {
                                            save::delete_save();
                                            self.board = data.board;
                                            self.score = data.score;
                                            self.current_piece = data.current_piece;
                                            self.current_x = data.current_x;
                                            self.current_y = data.current_y;
                                            self.current_rot = data.current_rot;
                                            self.next_piece = data.next_piece;
                                            self.hold_piece = data.hold_piece;
                                            self.bag = data.bag;
                                            self.can_hold = data.can_hold;
                                            self.tick_accum = data.tick_accum;
                                            self.lock_timer = data.lock_timer;
                                            self.lock_resets = data.lock_resets;
                                            self.game_mode = data.game_mode;
                                            self.state = GameState::Playing;
                                        }
                                        Err(_) => {
                                            save::delete_save();
                                            self.menu_has_continue = false;
                                        }
                                    }
                                }
                                1 => {
                                    self.mode_select_cursor = 0;
                                    self.state = GameState::ModeSelect;
                                }
                                2 => {
                                    self.menu_cursor = 0;
                                    self.state = GameState::Options;
                                }
                                _ => {
                                    save::save_pref(self.control_scheme, &self.options, self.game_mode, self.classic_high, self.relaxed_high);
                                    self.running = false;
                                }
                            }
                        }
                        Key::Escape => {
                            save::save_pref(self.control_scheme, &self.options, self.game_mode, self.classic_high, self.relaxed_high);
                            self.running = false;
                        }
                        _ => {}
                    }
                }
            }

            GameState::ModeSelect => {
                for k in input {
                    match k {
                        Key::Up => {
                            self.mode_select_cursor = self.mode_select_cursor.saturating_sub(1);
                        }
                        Key::Down => {
                            self.mode_select_cursor = (self.mode_select_cursor + 1).min(2);
                        }
                        Key::Enter => {
                            match self.mode_select_cursor {
                                0 => {
                                    self.game_mode = GameMode::Classic;
                                    self.start_game();
                                }
                                1 => {
                                    self.game_mode = GameMode::Relaxed;
                                    self.start_game();
                                }
                                _ => {
                                    self.state = GameState::Menu;
                                }
                            }
                        }
                        Key::Escape => {
                            self.state = GameState::Menu;
                        }
                        _ => {}
                    }
                }
            }

            GameState::Playing => {
                if self.clear_display_timer > 0.0 {
                    self.clear_display_timer -= dt;
                    self.particles.update(dt);
                    for k in input {
                        if matches!(key_to_action(self.control_scheme, *k), Some(crate::controls::Action::Pause)) {
                            self.paused_view = PausedView::Menu;
                            self.paused_cursor = 0;
                            self.state = GameState::Paused;
                            return;
                        }
                    }
                    if self.clear_display_timer <= 0.0 {
                        self.clear_display_timer = 0.0;
                        self.board.clear_lines_detailed();
                        if !self.clear_pending_cells.is_empty() {
                            self.particles.spawn_line_clear(
                                &self.clear_pending_cells,
                                self.options.gray_blocks,
                                true,
                            );
                        }
                        self.clear_rows.clear();
                        self.clear_piece_cells.clear();
                        self.clear_pending_cells.clear();
                    }
                    return;
                }

                for k in input {
                    match key_to_action(self.control_scheme, *k) {
                        Some(crate::controls::Action::MoveLeft) => self.try_move(-1),
                        Some(crate::controls::Action::MoveRight) => self.try_move(1),
                        Some(crate::controls::Action::RotateCW) => self.try_rotate(1),
                        Some(crate::controls::Action::RotateCCW) => self.try_rotate(-1),
                        Some(crate::controls::Action::SoftDrop) => self.soft_drop(),
                        Some(crate::controls::Action::HardDrop) => self.hard_drop(),
                        Some(crate::controls::Action::Hold) => self.hold(),
                        Some(crate::controls::Action::Pause) => {
                            self.paused_view = PausedView::Menu;
                            self.paused_cursor = 0;
                            self.state = GameState::Paused;
                            return;
                        }
                        None => {}
                    }
                }

                self.tick_accum += dt;
                let interval = match self.game_mode {
                    GameMode::Classic => self.score.tick_interval(),
                    GameMode::Relaxed => 1.0,
                };
                while self.tick_accum >= interval {
                    if !self
                        .board
                        .collides(self.current_piece, self.current_x, self.current_y + 1, self.current_rot)
                    {
                        self.current_y += 1;
                        self.tick_accum -= interval;
                    } else {
                        self.tick_accum = 0.0;
                        break;
                    }
                }

                if self
                    .board
                    .collides(self.current_piece, self.current_x, self.current_y + 1, self.current_rot)
                {
                    self.lock_timer += dt;
                    if self.lock_timer >= LOCK_DELAY {
                        self.lock_and_spawn();
                    }
                } else {
                    self.lock_timer = 0.0;
                    self.lock_resets = 0;
                }

                self.particles.update(dt);
            }

            GameState::Paused => {
                for k in input {
                    match k {
                        Key::Up => {
                            if self.paused_view == PausedView::Menu {
                                self.paused_cursor = self.paused_cursor.saturating_sub(1);
                            } else if self.paused_view == PausedView::Options {
                                self.menu_cursor = self.menu_cursor.saturating_sub(1);
                            }
                        }
                        Key::Down => {
                            if self.paused_view == PausedView::Menu {
                                self.paused_cursor = (self.paused_cursor + 1).min(4);
                            } else if self.paused_view == PausedView::Options {
                                self.menu_cursor = (self.menu_cursor + 1).min(Options::count());
                            }
                        }
                        Key::Enter => {
                            if self.paused_view == PausedView::Menu {
                                match self.paused_cursor {
                                    0 => self.state = GameState::Playing,
                                    1 => self.start_game(),
                                    2 => {
                                        self.menu_cursor = 0;
                                        self.paused_view = PausedView::Options;
                                    }
                                    3 => {
                                        if self.board.cells.iter().any(|r| r.iter().any(|c| c.is_some())) {
                                            let data = save::SaveData {
                                                board: self.board.clone(),
                                                score: self.score.clone(),
                                                current_piece: self.current_piece,
                                                current_x: self.current_x,
                                                current_y: self.current_y,
                                                current_rot: self.current_rot,
                                                next_piece: self.next_piece,
                                                hold_piece: self.hold_piece,
                                                bag: self.bag.clone(),
                                                can_hold: self.can_hold,
                                                tick_accum: self.tick_accum,
                                                lock_timer: self.lock_timer,
                                                lock_resets: self.lock_resets,
                                                game_mode: self.game_mode,
                                            };
                                            let _ = save::save_game(&data);
                                        }
                                        save::save_pref(self.control_scheme, &self.options, self.game_mode, self.classic_high, self.relaxed_high);
                                        self.state = GameState::Menu;
                                        self.menu_has_continue = save::save_exists();
                                        if self.menu_has_continue {
                                            if let Ok(data) = save::load_game() {
                                                self.saved_game_mode = data.game_mode;
                                            }
                                        }
                                    }
                                    4 => {
                                        if self.board.cells.iter().any(|r| r.iter().any(|c| c.is_some())) {
                                            let data = save::SaveData {
                                                board: self.board.clone(),
                                                score: self.score.clone(),
                                                current_piece: self.current_piece,
                                                current_x: self.current_x,
                                                current_y: self.current_y,
                                                current_rot: self.current_rot,
                                                next_piece: self.next_piece,
                                                hold_piece: self.hold_piece,
                                                bag: self.bag.clone(),
                                                can_hold: self.can_hold,
                                                tick_accum: self.tick_accum,
                                                lock_timer: self.lock_timer,
                                                lock_resets: self.lock_resets,
                                                game_mode: self.game_mode,
                                            };
                                            let _ = save::save_game(&data);
                                        }
                                        save::save_pref(self.control_scheme, &self.options, self.game_mode, self.classic_high, self.relaxed_high);
                                        self.running = false;
                                    }
                                    _ => {}
                                }
                            } else if self.paused_view == PausedView::Options {
                                if self.menu_cursor == Options::count() {
                                    self.paused_view = PausedView::Controls;
                                } else {
                                    self.options.toggle(self.menu_cursor);
                                }
                            }
                        }
                        Key::Left => {
                            if self.paused_view == PausedView::Controls {
                                self.control_scheme = self.control_scheme.prev();
                            }
                        }
                        Key::Right => {
                            if self.paused_view == PausedView::Controls {
                                self.control_scheme = self.control_scheme.next();
                            }
                        }
                        Key::P => {
                            if self.paused_view == PausedView::Controls {
                                self.paused_view = PausedView::Options;
                                self.menu_cursor = Options::count();
                            } else if self.paused_view == PausedView::Options {
                                self.paused_view = PausedView::Menu;
                            } else {
                                self.state = GameState::Playing;
                            }
                        }
                        Key::Escape => {
                            if self.paused_view == PausedView::Controls {
                                self.paused_view = PausedView::Options;
                                self.menu_cursor = Options::count();
                            } else if self.paused_view == PausedView::Options {
                                self.paused_view = PausedView::Menu;
                            } else {
                                self.state = GameState::Playing;
                            }
                        }
                        _ => {
                            if self.paused_view == PausedView::Controls {
                                self.paused_view = PausedView::Options;
                                self.menu_cursor = Options::count();
                            }
                        }
                    }
                }
            }

            GameState::Options => {
                for k in input {
                    if self.viewing_controls {
                        self.viewing_controls = false;
                        continue;
                    }
                    match k {
                        Key::Up => self.menu_cursor = self.menu_cursor.saturating_sub(1),
                        Key::Down => self.menu_cursor = (self.menu_cursor + 1).min(Options::count()),
                        Key::Enter => {
                            if self.menu_cursor == Options::count() {
                                self.viewing_controls = true;
                            } else {
                                self.options.toggle(self.menu_cursor);
                            }
                        }
                        Key::Escape => self.state = GameState::Menu,
                        _ => {}
                    }
                }
            }

            GameState::GameOver => {
                for k in input {
                    match k {
                        Key::Up => {
                            self.gameover_cursor = self.gameover_cursor.saturating_sub(1);
                        }
                        Key::Down => {
                            self.gameover_cursor = (self.gameover_cursor + 1).min(2);
                        }
                        Key::Enter => {
                            match self.gameover_cursor {
                                0 => self.start_game(),
                                1 => {
                                    self.state = GameState::Menu;
                                    self.menu_has_continue = save::save_exists();
                                    if self.menu_has_continue {
                                        if let Ok(data) = save::load_game() {
                                            self.saved_game_mode = data.game_mode;
                                        }
                                    }
                                }
                                _ => {
                                    save::save_pref(self.control_scheme, &self.options, self.game_mode, self.classic_high, self.relaxed_high);
                                    self.running = false;
                                }
                            }
                        }
                        Key::R => self.start_game(),
                        Key::Escape => {
                            self.state = GameState::Menu;
                            self.menu_has_continue = save::save_exists();
                            if self.menu_has_continue {
                                if let Ok(data) = save::load_game() {
                                    self.saved_game_mode = data.game_mode;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let current = match self.state {
            GameState::Playing | GameState::Paused => {
                Some((self.current_piece, self.current_x, self.current_y, self.current_rot))
            }
            _ => None,
        };
        let ghost = match (self.state, self.clear_display_timer > 0.0) {
            (GameState::Playing, false) => Some(self.board.ghost_y(
                self.current_piece,
                self.current_x,
                self.current_y,
                self.current_rot,
            )),
            _ => None,
        };
        let overlay = match self.state {
            GameState::Menu => Overlay::Menu {
                cursor: self.menu_cursor,
                continue_avail: self.menu_has_continue,
                saved_mode: self.saved_game_mode,
            },
            GameState::ModeSelect => Overlay::ModeSelect(self.mode_select_cursor),
            GameState::Paused => match self.paused_view {
                PausedView::Menu => Overlay::Paused(self.paused_cursor),
                PausedView::Options => Overlay::Options(self.menu_cursor),
                PausedView::Controls => Overlay::Controls(self.control_scheme),
            },
            GameState::Options => {
                if self.viewing_controls {
                    Overlay::Controls(self.control_scheme)
                } else {
                    Overlay::Options(self.menu_cursor)
                }
            }
            GameState::GameOver => Overlay::GameOver(self.gameover_cursor),
            _ => Overlay::None,
        };
        renderer::render(
            frame,
            &self.board,
            &self.score,
            overlay,
            current,
            ghost,
            self.next_piece,
            self.hold_piece,
            &self.options,
            &self.particles,
            self.game_mode,
            self.classic_high,
            self.relaxed_high,
            self.clear_display_timer,
            &self.clear_rows,
            self.clear_origin_col,
            &self.clear_piece_cells,
        );
    }

    fn running(&self) -> bool {
        self.running
    }

    fn name(&self) -> &str {
        "Tetrominal"
    }
}
