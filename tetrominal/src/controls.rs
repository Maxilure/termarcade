use termarcade_core::input::Key;

#[derive(Clone, Copy, PartialEq)]
pub enum ControlScheme {
    Arrows,
    Wasd,
    Hjkl,
}

impl ControlScheme {
    pub fn name(&self) -> &'static str {
        match self {
            ControlScheme::Arrows => "Arrows",
            ControlScheme::Wasd => "WASD",
            ControlScheme::Hjkl => "HJKL",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ControlScheme::Arrows => ControlScheme::Wasd,
            ControlScheme::Wasd => ControlScheme::Hjkl,
            ControlScheme::Hjkl => ControlScheme::Arrows,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            ControlScheme::Arrows => ControlScheme::Hjkl,
            ControlScheme::Wasd => ControlScheme::Arrows,
            ControlScheme::Hjkl => ControlScheme::Wasd,
        }
    }

    pub fn bindings(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            ControlScheme::Arrows => vec![
                ("\u{2190} \u{2192}", "Move"),
                ("\u{2191} / X", "Rotate CW"),
                ("Z", "Rotate CCW"),
                ("\u{2193}", "Soft Drop"),
                ("Space", "Hard Drop"),
                ("C", "Hold"),
                ("P", "Pause"),
                ("Esc", "Menu/Exit"),
            ],
            ControlScheme::Wasd => vec![
                ("A / D", "Move"),
                ("W", "Rotate CW"),
                ("Z", "Rotate CCW"),
                ("S", "Soft Drop"),
                ("Space", "Hard Drop"),
                ("C / X", "Hold"),
                ("P", "Pause"),
                ("Esc", "Menu/Exit"),
            ],
            ControlScheme::Hjkl => vec![
                ("H / L", "Move"),
                ("K", "Rotate CW"),
                ("Z", "Rotate CCW"),
                ("J", "Soft Drop"),
                ("Space", "Hard Drop"),
                ("C / X", "Hold"),
                ("P", "Pause"),
                ("Esc", "Menu/Exit"),
            ],
        }
    }
}

#[derive(Clone, Copy)]
pub enum Action {
    MoveLeft,
    MoveRight,
    RotateCW,
    RotateCCW,
    SoftDrop,
    HardDrop,
    Hold,
    Pause,
}

pub fn key_to_action(scheme: ControlScheme, key: Key) -> Option<Action> {
    use ControlScheme::*;
    use Action::*;

    match key {
        Key::Space => return Some(HardDrop),
        Key::P => return Some(Pause),
        Key::Escape => return Some(Pause),
        _ => {}
    }

    match (scheme, key) {
        (Arrows, Key::Left) => Some(MoveLeft),
        (Arrows, Key::Right) => Some(MoveRight),
        (Arrows, Key::Up) | (Arrows, Key::X) => Some(RotateCW),
        (Arrows, Key::Z) => Some(RotateCCW),
        (Arrows, Key::Down) => Some(SoftDrop),
        (Arrows, Key::Char('c') | Key::Char('C')) => Some(Hold),

        (Wasd, Key::Char('a') | Key::Char('A')) => Some(MoveLeft),
        (Wasd, Key::Char('d') | Key::Char('D')) => Some(MoveRight),
        (Wasd, Key::Char('w') | Key::Char('W')) => Some(RotateCW),
        (Wasd, Key::Z) => Some(RotateCCW),
        (Wasd, Key::Char('s') | Key::Char('S')) => Some(SoftDrop),
        (Wasd, Key::Char('c') | Key::Char('C')) | (Wasd, Key::X) => Some(Hold),

        (Hjkl, Key::Char('h') | Key::Char('H')) => Some(MoveLeft),
        (Hjkl, Key::Char('l') | Key::Char('L')) => Some(MoveRight),
        (Hjkl, Key::Char('k') | Key::Char('K')) => Some(RotateCW),
        (Hjkl, Key::Z) => Some(RotateCCW),
        (Hjkl, Key::Char('j') | Key::Char('J')) => Some(SoftDrop),
        (Hjkl, Key::Char('c') | Key::Char('C')) | (Hjkl, Key::X) => Some(Hold),

        _ => None,
    }
}
