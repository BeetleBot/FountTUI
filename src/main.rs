use clap::Parser;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, panic, time::Duration};

use fount::app::{App, ui::draw};
use fount::config::Cli;
use fount::export;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(export_path) = cli.export.clone() {
        let in_path = cli.files.first().cloned();

        if in_path.is_none() || !in_path.as_ref().unwrap().exists() {
            eprintln!("Error: Input file does not exist or was not provided.");
            std::process::exit(1);
        }

        let mut app = App::new(cli.clone());

        let fmt = cli.format.to_lowercase();
        let with_ansi = match fmt.as_str() {
            "ansi" => true,
            "ascii" => {
                app.config.force_ascii = true;
                false
            }
            "plain" => false,
            _ => {
                eprintln!(
                    "Error: Unknown export format '{}'. Expected 'plain', 'ascii', or 'ansi'.",
                    fmt
                );
                std::process::exit(1);
            }
        };

        app.cursor_y = usize::MAX;
        app.update_layout();

        let output = export::export_document(&app.layout, &app.lines, &app.config, with_ansi);

        if export_path.as_os_str() == "-" {
            print!("{}", output);
        } else if let Err(e) = std::fs::write(&export_path, output) {
            eprintln!("Error writing to '{}': {}", export_path.display(), e);
            std::process::exit(1);
        }

        return Ok(());
    }

    let default_panic = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let mut stdout = io::stdout();
        let (_, rows) = crossterm::terminal::size().unwrap_or((0, 24));
        let _ = execute!(stdout, crossterm::cursor::MoveTo(0, rows));

        let _ = disable_raw_mode();
        let _ = execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            LeaveAlternateScreen,
            DisableMouseCapture,
            PopKeyboardEnhancementFlags,
            crossterm::cursor::Show
        );
        println!();
        default_panic(info);
    }));

    let mut app = App::new(cli);

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    term.clear()?;

    let mut last_save = std::time::Instant::now();

    loop {
        term.draw(|f| draw(f, &mut app))?;

        let mut update_target_x = false;
        let mut text_changed = false;
        let mut cursor_moved = false;

        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            if app.handle_event(
                ev,
                &mut update_target_x,
                &mut text_changed,
                &mut cursor_moved,
            )? {
                break;
            }

            while event::poll(Duration::from_millis(0))? {
                let next_ev = event::read()?;
                if app.handle_event(
                    next_ev,
                    &mut update_target_x,
                    &mut text_changed,
                    &mut cursor_moved,
                )? {
                    break;
                }
            }
        }

        if app.config.auto_save
            && app.dirty
            && app.file.is_some()
            && last_save.elapsed() >= Duration::from_secs(app.config.auto_save_interval)
        {
            if let Err(e) = app.save() {
                app.set_status(&format!("Auto-save failed: {}", e));
            } else {
                app.set_status("Auto-saved");
            }
            last_save = std::time::Instant::now();
        }

        if text_changed {
            app.parse_document();
        }

        if text_changed || cursor_moved {
            app.update_autocomplete();
            app.update_layout();
        }

        if update_target_x {
            app.target_visual_x = app.current_visual_x();
        }
    }

    let (_, rows) = crossterm::terminal::size().unwrap_or((0, 24));

    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste,
        PopKeyboardEnhancementFlags
    )?;

    if std::env::var("TERM").unwrap_or_default() == "linux" {
        execute!(
            term.backend_mut(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, rows)
        )?;
    }

    term.show_cursor()?;
    Ok(())
}
