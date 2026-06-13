use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use std::io::{stdout, Write};

pub fn init() -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, Hide)?;

    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore();
        prev(info);
    }));

    Ok(())
}

pub fn clear_screen() -> std::io::Result<()> {
    use crossterm::queue;
    let mut out = stdout();
    queue!(out, Clear(ClearType::All))?;
    out.flush()?;
    Ok(())
}

pub fn restore() -> std::io::Result<()> {
    use crossterm::style::ResetColor;
    let mut out = stdout();
    execute!(out, Show, ResetColor, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
