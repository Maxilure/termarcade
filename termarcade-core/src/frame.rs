use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use std::io::{stdout, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
}

impl Style {
    pub const fn new(fg: Color, bg: Color) -> Self {
        Style { fg, bg }
    }

    pub const fn reset() -> Self {
        Style {
            fg: Color::Reset,
            bg: Color::Reset,
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::reset()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub char: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            char: ' ',
            style: Style::reset(),
        }
    }
}

impl Cell {
    pub fn new(char: char, style: Style) -> Self {
        Cell { char, style }
    }
}

pub struct Frame {
    cells: Vec<Cell>,
    prev: Vec<Cell>,
    width: u16,
    height: u16,
    force_redraw: bool,
}

impl Frame {
    pub fn new(width: u16, height: u16) -> Self {
        let len = (width as usize) * (height as usize);
        Frame {
            cells: vec![Cell::default(); len],
            prev: vec![Cell::default(); len],
            width,
            height,
            force_redraw: false,
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    fn idx(&self, x: u16, y: u16) -> usize {
        y as usize * self.width as usize + x as usize
    }

    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if x < self.width && y < self.height {
            let i = self.idx(x, y);
            self.cells[i] = cell;
        }
    }

    pub fn text(&mut self, x: u16, y: u16, s: &str, style: Style) {
        for (i, c) in s.chars().enumerate() {
            let cell = Cell {
                char: c,
                style: style,
            };
            self.set(x + i as u16, y, cell);
        }
    }

    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, cell: Cell) {
        for dy in 0..h {
            for dx in 0..w {
                self.set(x + dx, y + dy, cell);
            }
        }
    }

    pub fn clear(&mut self) {
        for c in &mut self.cells {
            *c = Cell::default();
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let len = (width as usize) * (height as usize);
        self.cells = vec![Cell::default(); len];
        self.prev = vec![Cell::default(); len];
        self.width = width;
        self.height = height;
        self.force_redraw = true;
    }

    pub fn take_force_redraw(&mut self) -> bool {
        if self.force_redraw {
            self.force_redraw = false;
            true
        } else {
            false
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        let mut out = stdout();
        let mut any_changed = false;
        let force = self.force_redraw;
        self.force_redraw = false;

        for y in 0..self.height {
            for x in 0..self.width {
                let i = self.idx(x, y);
                if force || self.cells[i] != self.prev[i] {
                    if !any_changed {
                        any_changed = true;
                    }
                    queue!(
                        out,
                        MoveTo(x, y),
                        SetForegroundColor(self.cells[i].style.fg),
                        SetBackgroundColor(self.cells[i].style.bg),
                        Print(self.cells[i].char),
                    )?;
                }
            }
        }

        if any_changed {
            queue!(out, ResetColor)?;
            out.flush()?;
        }

        std::mem::swap(&mut self.cells, &mut self.prev);
        for c in &mut self.cells {
            *c = Cell::default();
        }

        Ok(())
    }
}
