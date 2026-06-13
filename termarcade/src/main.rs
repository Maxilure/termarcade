mod menu;

use menu::TermArcadeMenu;
use termarcade_core::game::run_loop;
use termarcade_core::terminal;

fn print_help() {
    println!("TermArcade — Terminal Arcade Launcher");
    println!();
    println!("USAGE:");
    println!("    termarcade [SUBCOMMAND]");
    println!("    termarcade tetrominal [MODE]");
    println!();
    println!("SUBCOMMANDS:");
    println!("    tetrominal           Open Tetrominal to its menu");
    println!("    tetrominal c         Open Tetrominal in Classic mode");
    println!("    tetrominal r         Open Tetrominal in Relaxed mode");
    println!("    list                 List available games");
    println!("    help, -h, --help     Show this help");
    println!("    version, -v, --version");
    println!("                        Show version");
    println!();
    println!("EXAMPLES:");
    println!("    termarcade                  Launch the arcade menu");
    println!("    termarcade tetrominal       Open Tetrominal to its menu");
    println!("    termarcade tetrominal c     Start Classic mode immediately");
    println!("    termarcade tetrominal r     Start Relaxed mode immediately");
    println!("    termarcade list             List available games");
    println!("    termarcade -h               Show this help");
}

fn print_version() {
    println!("termarcade {}", env!("CARGO_PKG_VERSION"));
}

fn print_list() {
    println!("Available games:");
    println!("  Tetrominal    Stack blocks, clear lines, try not to lose");
}

fn run_arcade_menu() -> std::io::Result<()> {
    terminal::init()?;

    let mut arcade_menu = TermArcadeMenu::new();

    'outer: loop {
        run_loop(&mut arcade_menu)?;

        match arcade_menu.take_selection() {
            Some("Tetrominal") => {
                let mut game = tetrominal::game::BlockDropGame::new();
                run_loop(&mut game)?;
            }
            None => break 'outer,
            _ => {}
        }

        arcade_menu.reset();
    }

    terminal::restore()?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        None => run_arcade_menu(),
        Some("help") | Some("-h") | Some("--help") => {
            print_help();
            Ok(())
        }
        Some("version") | Some("-v") | Some("--version") => {
            print_version();
            Ok(())
        }
        Some("list") => {
            print_list();
            Ok(())
        }
        Some("tetrominal") => {
            match args.get(2).map(|s| s.as_str()) {
                Some(m) if m != "c" && m != "r" => {
                    eprintln!("error: unknown mode '{}'", m);
                    eprintln!("usage: termarcade tetrominal [c|r]");
                    std::process::exit(1);
                }
                _ => {}
            }
            terminal::init()?;
            match args.get(2).map(|s| s.as_str()) {
                Some("c") => {
                    let mut game =
                        tetrominal::game::BlockDropGame::with_mode(tetrominal::game::GameMode::Classic);
                    run_loop(&mut game)?;
                }
                Some("r") => {
                    let mut game =
                        tetrominal::game::BlockDropGame::with_mode(tetrominal::game::GameMode::Relaxed);
                    run_loop(&mut game)?;
                }
                None => {
                    let mut game = tetrominal::game::BlockDropGame::new();
                    run_loop(&mut game)?;
                }
                _ => unreachable!(),
            }
            terminal::restore()?;
            Ok(())
        }
        Some(other) => {
            eprintln!("error: unknown subcommand '{}'", other);
            eprintln!("usage: termarcade [help|list|tetrominal]");
            std::process::exit(1);
        }
    }
}
