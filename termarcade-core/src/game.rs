use crate::frame::Frame;
use crate::input::{Input, Key};
use crate::terminal;
use std::time::{Duration, Instant};

const TICK_RATE: Duration = Duration::from_nanos(16_666_667);
const MAX_FRAME_TIME: Duration = Duration::from_nanos(100_000_000);

pub trait Game {
    fn tick(&mut self, dt: Duration, input: &[Key]);
    fn draw(&self, frame: &mut Frame);
    fn running(&self) -> bool;
    fn name(&self) -> &str;
}

pub fn run(game: &mut impl Game) -> std::io::Result<()> {
    terminal::init()?;
    let result = run_loop(game);
    terminal::restore()?;
    result
}

pub fn run_loop(game: &mut impl Game) -> std::io::Result<()> {
    let (mut w, mut h) = crossterm::terminal::size()?;
    terminal::clear_screen()?;
    let mut frame = Frame::new(w, h);
    let mut input = Input::new();
    let mut last = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now - last;
        last = now;

        let dt = if dt > MAX_FRAME_TIME { TICK_RATE } else { dt };

        if let Ok((nw, nh)) = crossterm::terminal::size() {
            if nw != w || nh != h {
                w = nw;
                h = nh;
                frame.resize(w, h);
            }
        }

        input.poll();

        if !game.running() {
            break Ok(());
        }

        game.tick(dt, input.events());
        game.draw(&mut frame);

        if let Err(e) = frame.flush() {
            break Err(e);
        }

        let elapsed = now.elapsed();
        if elapsed < TICK_RATE {
            std::thread::sleep(TICK_RATE - elapsed);
        }
    }
}
