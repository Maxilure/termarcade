#[derive(Clone, Copy, PartialEq)]
pub struct Options {
    pub gray_blocks: bool,
    pub ghost_piece: bool,
    pub grid_lines: bool,
    pub flash_on_clear: bool,
    pub hard_drop_trail: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            gray_blocks: false,
            ghost_piece: true,
            grid_lines: false,
            flash_on_clear: true,
            hard_drop_trail: true,
        }
    }
}

impl Options {
    pub fn toggle(&mut self, index: usize) {
        match index {
            0 => self.gray_blocks = !self.gray_blocks,
            1 => self.ghost_piece = !self.ghost_piece,
            2 => self.grid_lines = !self.grid_lines,
            3 => self.flash_on_clear = !self.flash_on_clear,
            4 => self.hard_drop_trail = !self.hard_drop_trail,
            _ => {}
        }
    }

    pub fn count() -> usize {
        5
    }

    pub fn label(index: usize) -> &'static str {
        match index {
            0 => "Gray Blocks",
            1 => "Ghost Piece",
            2 => "Grid Lines",
            3 => "Pop on Clear",
            4 => "Hard Drop Trail",
            _ => "",
        }
    }

    pub fn val(&self, index: usize) -> &'static str {
        let v = match index {
            0 => self.gray_blocks,
            1 => self.ghost_piece,
            2 => self.grid_lines,
            3 => self.flash_on_clear,
            4 => self.hard_drop_trail,
            _ => return "",
        };
        if v { " On" } else { "Off" }
    }
}
