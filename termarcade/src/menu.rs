use crossterm::style::Color;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use termarcade_core::frame::{Cell, Frame, Style};
use termarcade_core::game::Game;
use termarcade_core::input::Key;

const BANNERS: &[&[&str]] = &[
    &[
        "████████╗███████╗██████╗ ███╗   ███╗ █████╗ ██████╗  ██████╗ █████╗ ██████╗ ███████╗",
        "╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██╔══██╗██╔══██╗██╔════╝██╔══██╗██╔══██╗██╔════╝",
        "   ██║   █████╗  ██████╔╝██╔████╔██║███████║██████╔╝██║     ███████║██║  ██║█████╗",
        "   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██╔══██║██╔══██╗██║     ██╔══██║██║  ██║██╔══╝",
        "   ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║  ██║██║  ██║╚██████╗██║  ██║██████╔╝███████╗",
        "   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚═════╝ ╚══════╝",
    ],
];

pub struct GameEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
}

const NUM_PARTICLES: usize = 30;

struct Particle {
    angle: f64,
    radius: f64,
    speed: f64,
    ch: char,
    color: Color,
}

const GLITCH_CHARS: &[char] = &['█', '▓', '░', '▒', '╔', '╗', '╚', '╝', '║', '═', '▐', '▌', '◆', '●', '○', '■'];

#[derive(Clone, Copy)]
enum BannerEffect {
    Glitch,
    Typewriter,
    MatrixDrip,
}

const NUM_BANNER_EFFECTS: usize = 3;

fn lcg_seed(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed
}

pub struct TermArcadeMenu {
    running: bool,
    cursor: usize,
    pub games: &'static [GameEntry],
    selected: Option<&'static str>,
    banner_idx: usize,
    particles: Vec<Particle>,
    effect: BannerEffect,
    effect_time: f64,
    drip_starts: Vec<f64>,
    particle_alpha: f64,
}

impl TermArcadeMenu {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as usize;
        let banner_idx = now % BANNERS.len();

        let mut seed = (now as u64) ^ 0x9e3779b9;
        let particles = (0..NUM_PARTICLES)
            .map(|_| {
                let s = lcg_seed(&mut seed);
                let r = (s >> 32) as u64 | 1;
                Particle {
                    angle: (s as f64 / u64::MAX as f64) * std::f64::consts::PI * 2.0,
                    radius: 0.8 + (r % 5) as f64 * 0.1,
                    speed: -3.0 + ((s >> 16) % 60) as f64 * 0.1,
                    ch: match s % 4 { 0 => '·', 1 => '•', 2 => '∘', _ => '.' },
                    color: match (s >> 8) % 4 {
                        0 => Color::White,
                        1 => Color::Cyan,
                        2 => Color::Cyan,
                        _ => Color::DarkCyan,
                    },
                }
            })
            .collect();

        let effect_idx = (seed >> 20) % NUM_BANNER_EFFECTS as u64;
        let effect = match effect_idx {
            0 => BannerEffect::Glitch,
            1 => BannerEffect::Typewriter,
            _ => BannerEffect::MatrixDrip,
        };

        let banner_w = BANNERS[banner_idx].iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0);
        let drip_starts: Vec<f64> = (0..banner_w)
            .map(|_| {
                let s = lcg_seed(&mut seed);
                (s as f64 / u64::MAX as f64) * 1.5
            })
            .collect();

        Self {
            running: true,
            cursor: 0,
            games: &[
                GameEntry {
                    id: "Tetrominal",
                    name: "Tetrominal",
                    desc: "Stack blocks, clear lines, try not to lose",
                },
            ],
            selected: None,
            banner_idx,
            particles,
            effect,
            effect_time: 0.0,
            drip_starts,
            particle_alpha: 0.0,
        }
    }

    pub fn take_selection(&mut self) -> Option<&'static str> {
        self.selected.take()
    }

    pub fn reset(&mut self) {
        self.running = true;
        self.cursor = 0;
        self.selected = None;
    }
}

impl Game for TermArcadeMenu {
    fn tick(&mut self, dt: Duration, input: &[Key]) {
        let dt = dt.as_secs_f64();
        self.effect_time += dt;

        let banner_w = BANNERS[self.banner_idx].iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as f64;
        let finished = match self.effect {
            BannerEffect::Glitch => self.effect_time * 30.0 >= banner_w,
            BannerEffect::Typewriter => self.effect_time * 40.0 >= banner_w,
            BannerEffect::MatrixDrip => {
                let latest = self.drip_starts.iter().copied().fold(0.0f64, f64::max);
                self.effect_time > latest + 1.0
            }
        };
        if finished {
            self.particle_alpha = (self.particle_alpha + dt * 1.5).min(1.0);
        }

        for p in &mut self.particles {
            p.angle += p.speed * dt;
            if p.angle < 0.0 {
                p.angle += std::f64::consts::PI * 2.0;
            }
            if p.angle > std::f64::consts::PI * 2.0 {
                p.angle -= std::f64::consts::PI * 2.0;
            }
        }
        for k in input {
            match k {
                Key::Up => {
                    self.cursor = self.cursor.saturating_sub(1);
                }
                Key::Down => {
                    self.cursor = (self.cursor + 1).min(self.games.len() - 1);
                }
                Key::Enter => {
                    self.selected = Some(self.games[self.cursor].id);
                    self.running = false;
                }
                Key::Escape => {
                    self.running = false;
                }
                _ => {}
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let fw = frame.width();
        let fh = frame.height();

        let banner_lines = BANNERS[self.banner_idx];

        let banner_w = banner_lines.iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as u16;

        let game = &self.games[self.cursor];
        let desc_lines: Vec<&str> = game.desc.split('\n').collect();
        let desc_h = desc_lines.len() as u16;

        let entry_w = banner_w.max(40);
        let ox = (fw.saturating_sub(entry_w)) / 2;

        // Banner rendering based on effect
        let base_y: u16 = 4;
        let mut banner_cells = std::collections::HashSet::new();
        let mut cy: u16 = base_y + 6;

        // Add base banner cells for particle collision
        for (line_idx, line) in banner_lines.iter().enumerate() {
            for (i, _) in line.chars().enumerate() {
                banner_cells.insert((ox + i as u16, base_y + line_idx as u16));
            }
        }

        match self.effect {
            BannerEffect::Glitch => {
                let resolve_col = (self.effect_time * 30.0) as usize;
                let mut seed = (self.effect_time * 1000.0) as u64;
                for (line_idx, line) in banner_lines.iter().enumerate() {
                    let y = base_y + line_idx as u16;
                    let mut rendered = String::new();
                    for (i, ch) in line.chars().enumerate() {
                        if i < resolve_col {
                            rendered.push(ch);
                        } else {
                            let s = lcg_seed(&mut seed);
                            rendered.push(GLITCH_CHARS[(s % GLITCH_CHARS.len() as u64) as usize]);
                        }
                    }
                    frame.text(ox, y, &rendered, Style::new(Color::Cyan, Color::Reset));
                }
            }
            BannerEffect::Typewriter => {
                let reveal_col = (self.effect_time * 40.0) as usize;
                for (line_idx, line) in banner_lines.iter().enumerate() {
                    let y = base_y + line_idx as u16;
                    for (i, ch) in line.chars().enumerate() {
                        if i <= reveal_col {
                            frame.set(ox + i as u16, y, Cell::new(ch, Style::new(Color::Cyan, Color::Reset)));
                        }
                    }
                }
            }
            BannerEffect::MatrixDrip => {
                let drip_speed = 12.0;
                for (line_idx, line) in banner_lines.iter().enumerate() {
                    let y = base_y + line_idx as u16;
                    let mut rendered = String::new();
                    for (i, ch) in line.chars().enumerate() {
                        let drip_row = if i < self.drip_starts.len() {
                            if self.effect_time > self.drip_starts[i] {
                                ((self.effect_time - self.drip_starts[i]) * drip_speed) as i16 - 2
                            } else {
                                -100
                            }
                        } else {
                            -100
                        };
                        if drip_row >= y as i16 {
                            rendered.push(ch);
                        } else {
                            let mut cs = (i as u64).wrapping_mul(31).wrapping_add(line_idx as u64 * 17).wrapping_add((self.effect_time * 100.0) as u64);
                            let s = lcg_seed(&mut cs);
                            rendered.push(GLITCH_CHARS[(s % GLITCH_CHARS.len() as u64) as usize]);
                        }
                    }
                    frame.text(ox, y, &rendered, Style::new(Color::Cyan, Color::Reset));
                }
            }
        }

        // Particles orbiting the banner in an ellipse that matches its shape
        let cxx = ox as f64 + banner_w as f64 / 2.0;
        let cyy = 4.0 + 3.0;
        let rx = banner_w as f64 / 2.0 + 3.0;
        let ry = 3.0 + 3.0;
        for (i, p) in self.particles.iter().enumerate() {
            let threshold = i as f64 / self.particles.len() as f64;
            if self.particle_alpha > threshold {
                let px = (cxx + rx * p.radius * p.angle.cos()).round() as u16;
                let py = (cyy + ry * p.radius * p.angle.sin()).round() as u16;
                if !banner_cells.contains(&(px, py)) {
                    frame.set(px, py, Cell::new(p.ch, Style::new(p.color, Color::Reset)));
                }
            }
        }

        // Games + footer centered in the remaining vertical space
        let content_h = 1 + desc_h + 1 + 1;
        let after_banner = cy + 1;
        let remaining = fh.saturating_sub(after_banner);
        cy = after_banner + (remaining.saturating_sub(content_h)) / 2;

        for (i, game) in self.games.iter().enumerate() {
            let selected = i == self.cursor;
            let icon = if selected { "▸" } else { "·" };
            let fg = if selected { Color::White } else { Color::DarkGrey };

            let entry_line = format!("{}  {}", icon, game.name);
            let entry_len = entry_line.chars().count() as u16;
            let ex = ox + (entry_w.saturating_sub(entry_len)) / 2;
            frame.text(ex, cy, &entry_line, Style::new(fg, Color::Reset));
            cy += 1;

            if selected {
                for dl in &desc_lines {
                    let dl_len = dl.chars().count() as u16;
                    let dx = ox + (entry_w.saturating_sub(dl_len)) / 2;
                    frame.text(dx, cy, dl, Style::new(Color::DarkGrey, Color::Reset));
                    cy += 1;
                }
            }
        }

        cy += 1;

        let footer = "↑↓ Select    Enter Play    Esc Quit";
        let footer_len = footer.chars().count() as u16;
        let fx = ox + (entry_w.saturating_sub(footer_len)) / 2;
        frame.text(fx, cy, footer, Style::new(Color::DarkGrey, Color::Reset));
    }

    fn running(&self) -> bool {
        self.running
    }

    fn name(&self) -> &str {
        "TermArcade"
    }
}
