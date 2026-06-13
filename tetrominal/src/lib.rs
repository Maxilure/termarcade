pub mod board;
pub mod collision;
pub mod controls;
pub mod game;
pub mod options;
pub mod particles;
pub mod renderer;
pub mod save;
pub mod score;
pub mod tetromino;

use termarcade_core::game::run;

pub fn run_game() -> std::io::Result<()> {
    let mut game = crate::game::BlockDropGame::new();
    run(&mut game)
}
