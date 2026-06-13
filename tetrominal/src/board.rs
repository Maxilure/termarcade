use crate::tetromino::PieceType;

pub const BOARD_W: usize = 10;
pub const BOARD_H: usize = 20;
pub const BOARD_BUFFER: usize = 4;
pub const TOTAL_H: usize = BOARD_H + BOARD_BUFFER;

#[derive(Clone)]
pub struct Board {
    pub cells: Vec<[Option<PieceType>; BOARD_W]>,
    pub clear_count: u32,
}

impl Board {
    pub fn new() -> Self {
        Board {
            cells: vec![[None; BOARD_W]; TOTAL_H],
            clear_count: 0,
        }
    }

    pub fn reset(&mut self) {
        for row in &mut self.cells {
            *row = [None; BOARD_W];
        }
        self.clear_count = 0;
    }

    pub fn collides(&self, piece: PieceType, x: i16, y: i16, rot: usize) -> bool {
        let shape = piece.shape(rot);
        for (ri, &mask) in shape.iter().enumerate() {
            if mask == 0 {
                continue;
            }
            let row = y + ri as i16;
            if row >= TOTAL_H as i16 {
                return true;
            }
            for col in 0..16u16 {
                if mask & (1 << col) != 0 {
                    let cx = x + col as i16;
                    if cx < 0 || cx >= BOARD_W as i16 {
                        return true;
                    }
                    let cy = row;
                    if cy >= 0 && self.cells[cy as usize][cx as usize].is_some() {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn lock(&mut self, piece: PieceType, x: i16, y: i16, rot: usize) {
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
                        self.cells[row as usize][cx as usize] = Some(piece);
                    }
                }
            }
        }
    }

    pub fn capture_cleared_cells(&self) -> Vec<(usize, usize, PieceType)> {
        let mut cells = Vec::new();
        for row in 0..TOTAL_H {
            if self.cells[row].iter().all(|c| c.is_some()) {
                for col in 0..BOARD_W {
                    if let Some(pt) = self.cells[row][col] {
                        let vis_row = row.saturating_sub(BOARD_BUFFER);
                        cells.push((col, vis_row, pt));
                    }
                }
            }
        }
        cells
    }

    pub fn find_full_rows(&self) -> Vec<usize> {
        let mut rows = Vec::new();
        for i in 0..TOTAL_H {
            if self.cells[i].iter().all(|c| c.is_some()) {
                rows.push(i);
            }
        }
        rows
    }

    pub fn clear_lines_detailed(&mut self) -> (u32, Vec<usize>) {
        let rows = self.find_full_rows();
        let cleared = rows.len() as u32;
        if cleared == 0 {
            return (0, Vec::new());
        }

        let mut write_idx = TOTAL_H - 1;
        for read_idx in (0..TOTAL_H).rev() {
            if self.cells[read_idx].iter().all(|c| c.is_some()) {
                continue;
            }
            if write_idx != read_idx {
                self.cells[write_idx] = self.cells[read_idx];
            }
            if write_idx > 0 {
                write_idx -= 1;
            }
        }
        for i in (0..=write_idx).rev() {
            self.cells[i] = [None; BOARD_W];
        }

        self.clear_count += cleared;
        (cleared, rows)
    }

    pub fn ghost_y(&self, piece: PieceType, x: i16, y: i16, rot: usize) -> i16 {
        let mut gy = y;
        while !self.collides(piece, x, gy + 1, rot) {
            gy += 1;
        }
        gy
    }
}
