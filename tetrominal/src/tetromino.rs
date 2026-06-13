use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

pub const PIECE_TYPES: [PieceType; 7] = [
    PieceType::I,
    PieceType::O,
    PieceType::T,
    PieceType::S,
    PieceType::Z,
    PieceType::J,
    PieceType::L,
];

impl PieceType {
    pub fn shape(self, rotation: usize) -> [u16; 4] {
        SHAPES[self as usize][rotation % 4]
    }

    pub fn color(self) -> Color {
        COLORS[self as usize]
    }

    pub fn spawn_x(self) -> i16 {
        3
    }

    pub fn spawn_y(self) -> i16 {
        const BUF: i16 = crate::board::BOARD_BUFFER as i16;
        match self {
            PieceType::I => BUF - 1,
            _ => BUF,
        }
    }
}

pub const COLORS: [Color; 7] = [
    Color::Cyan,
    Color::Yellow,
    Color::Magenta,
    Color::Green,
    Color::Red,
    Color::Blue,
    Color::White,
];

const SHAPES: [[[u16; 4]; 4]; 7] = [
    // I
    [
        [0b0000, 0b1111, 0b0000, 0b0000],
        [0b0100, 0b0100, 0b0100, 0b0100],
        [0b0000, 0b0000, 0b1111, 0b0000],
        [0b0010, 0b0010, 0b0010, 0b0010],
    ],
    // O
    [
        [0b0110, 0b0110, 0b0000, 0b0000],
        [0b0110, 0b0110, 0b0000, 0b0000],
        [0b0110, 0b0110, 0b0000, 0b0000],
        [0b0110, 0b0110, 0b0000, 0b0000],
    ],
    // T
    [
        [0b010, 0b111, 0b000, 0b000],
        [0b010, 0b011, 0b010, 0b000],
        [0b000, 0b111, 0b010, 0b000],
        [0b010, 0b110, 0b010, 0b000],
    ],
    // S
    [
        [0b011, 0b110, 0b000, 0b000],
        [0b010, 0b011, 0b001, 0b000],
        [0b000, 0b011, 0b110, 0b000],
        [0b100, 0b110, 0b010, 0b000],
    ],
    // Z
    [
        [0b110, 0b011, 0b000, 0b000],
        [0b001, 0b011, 0b010, 0b000],
        [0b000, 0b110, 0b011, 0b000],
        [0b010, 0b110, 0b100, 0b000],
    ],
    // J
    [
        [0b100, 0b111, 0b000, 0b000],
        [0b011, 0b010, 0b010, 0b000],
        [0b000, 0b111, 0b001, 0b000],
        [0b010, 0b010, 0b110, 0b000],
    ],
    // L
    [
        [0b001, 0b111, 0b000, 0b000],
        [0b010, 0b010, 0b011, 0b000],
        [0b000, 0b111, 0b100, 0b000],
        [0b110, 0b010, 0b010, 0b000],
    ],
];
