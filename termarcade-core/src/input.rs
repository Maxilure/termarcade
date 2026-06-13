use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Z,
    X,
    Space,
    Enter,
    Escape,
    P,
    R,
    Char(char),
    Unknown,
}

pub struct Input {
    events: Vec<Key>,
}

impl Input {
    pub fn new() -> Self {
        Input { events: Vec::new() }
    }

    pub fn poll(&mut self) {
        self.events.clear();
        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Event::Key(ke) = event::read().unwrap() {
                if ke.kind == KeyEventKind::Press {
                    self.events.push(code_to_key(ke));
                }
            }
        }
    }

    pub fn events(&self) -> &[Key] {
        &self.events
    }
}

fn code_to_key(ke: KeyEvent) -> Key {
    match ke.code {
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Char('z') | KeyCode::Char('Z') => Key::Z,
        KeyCode::Char('x') | KeyCode::Char('X') => Key::X,
        KeyCode::Char(' ') => Key::Space,
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Escape,
        KeyCode::Char('p') | KeyCode::Char('P') => Key::P,
        KeyCode::Char('r') | KeyCode::Char('R') => Key::R,
        KeyCode::Char(c) => Key::Char(c),
        _ => Key::Unknown,
    }
}
