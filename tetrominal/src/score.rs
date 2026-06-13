const LINE_SCORES: [u32; 5] = [0, 100, 300, 500, 800];

const GRAVITY_TABLE: [f64; 30] = [
    1.000, 0.793, 0.618, 0.473, 0.355, 0.262, 0.190, 0.135, 0.094, 0.064,
    0.043, 0.028, 0.018, 0.011, 0.007, 0.004, 0.002, 0.001, 0.001, 0.001,
    0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001,
];

#[derive(Clone)]
pub struct ScoreManager {
    pub score: u32,
    pub level: u32,
    pub lines: u32,
}

impl ScoreManager {
    pub fn new() -> Self {
        ScoreManager {
            score: 0,
            level: 0,
            lines: 0,
        }
    }

    pub fn reset(&mut self) {
        self.score = 0;
        self.level = 0;
        self.lines = 0;
    }

    pub fn add_lines(&mut self, n: u32) {
        let idx = (n as usize).min(4);
        self.score += LINE_SCORES[idx] * (self.level + 1);
        self.lines += n;
        let new_level = self.lines / 10;
        if new_level != self.level {
            self.level = new_level;
        }
    }

    pub fn add_soft_drop(&mut self) {
        self.score += 1;
    }

    pub fn add_hard_drop(&mut self, distance: u32) {
        self.score += distance * 2;
    }

    pub fn tick_interval(&self) -> f64 {
        let idx = (self.level as usize).min(GRAVITY_TABLE.len() - 1);
        GRAVITY_TABLE[idx]
    }
}
