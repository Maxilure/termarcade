use crossterm::style::Color;
use rand::Rng;

use termarcade_core::frame::{Cell, Frame, Style};

use crate::tetromino::PieceType;

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    max_life: f64,
    ch: char,
    color: Color,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn update(&mut self, dt: f64) {
        let gravity = 15.0;
        self.particles.retain_mut(|p| {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += gravity * dt;
            p.life -= dt;
            p.life > 0.0
        });
    }

    pub fn spawn_line_clear(&mut self, cleared_cells: &[(usize, usize, PieceType)], gray: bool, big: bool) {
        let mut rng = rand::thread_rng();
        let normal_chars = ['·', '*', '░', '+'];
        let sparkle_chars = ['✦', '✧', '★'];
        for &(col, row, pt) in cleared_cells {
            let color = if gray { Color::DarkGrey } else { pt.color() };
            let n = if big { rng.gen_range(5..=7) } else { rng.gen_range(2..=3) };
            for _ in 0..n {
                let ch = if big && rng.gen_bool(0.65) {
                    sparkle_chars[rng.gen_range(0..sparkle_chars.len())]
                } else {
                    normal_chars[rng.gen_range(0..normal_chars.len())]
                };
                self.particles.push(Particle {
                    x: col as f64 + 0.5,
                    y: row as f64 + 0.5,
                    vx: rng.gen_range(if big { -5.0..5.0 } else { -3.0..3.0 }),
                    vy: rng.gen_range(if big { -16.0..-7.0 } else { -12.0..-5.0 }),
                    life: rng.gen_range(if big { 0.5..0.9 } else { 0.3..0.7 }),
                    max_life: if big { 0.9 } else { 0.7 },
                    ch,
                    color,
                });
            }
        }
    }

    pub fn spawn_hard_drop_trail(
        &mut self,
        piece: PieceType,
        rot: usize,
        px: i16,
        start_y: i16,
        end_y: i16,
        gray: bool,
    ) {
        let mut rng = rand::thread_rng();
        let chars = ['·', ':', '⋮', '·'];
        let color = if gray { Color::DarkGrey } else { piece.color() };
        let shape = piece.shape(rot);

        let cells: Vec<(i16, i16)> = {
            let mut v = Vec::new();
            for (ri, &mask) in shape.iter().enumerate() {
                for col in 0..16u16 {
                    if mask & (1 << col) != 0 {
                        v.push((col as i16, ri as i16));
                    }
                }
            }
            v
        };

        if cells.is_empty() {
            return;
        }

        const BUF: i16 = 4;

        for abs_row in start_y..end_y {
            let n = rng.gen_range(1..=cells.len().min(3));
            for _ in 0..n {
                let (co, ri) = cells[rng.gen_range(0..cells.len())];
                let vis_row = abs_row + ri - BUF;
                if vis_row < 0 {
                    continue;
                }
                let cx = (px + co) as f64 + rng.gen_range(-0.3..0.3);
                let cy = vis_row as f64 + rng.gen_range(-0.3..0.3);
                self.particles.push(Particle {
                    x: cx,
                    y: cy,
                    vx: rng.gen_range(-2.0..2.0),
                    vy: rng.gen_range(1.0..4.0),
                    life: rng.gen_range(0.2..0.5),
                    max_life: 0.5,
                    ch: chars[rng.gen_range(0..chars.len())],
                    color,
                });
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, board_x: u16, board_y: u16, cell_w: u16, cell_h: u16) {
        for p in &self.particles {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            if alpha < 0.1 {
                continue;
            }
            let sx = board_x + (p.x * cell_w as f64) as u16;
            let sy = board_y + (p.y * cell_h as f64) as u16;
            let style = Style::new(p.color, Color::Reset);
            frame.set(sx, sy, Cell::new(p.ch, style));
        }
    }
}
