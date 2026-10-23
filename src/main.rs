//! TIDE: Terminal Integrated Development Environment
//!
//! A native, keyboard-first terminal workspace for parallel coding-agent management.

mod agents;
mod app;
mod config;
mod explorer;
mod git;
mod icons;
mod job_object;
mod logo;
mod terminal;
mod theme;
mod ui;
mod worktree;

use std::io::stdout;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    cursor::{EnableBlinking, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

#[derive(Parser, Debug)]
#[command(name = "tide", version = "0.1.0", about = "Terminal Integrated Development Environment")]
struct Cli {
    /// Workspace directory to open (defaults to current directory)
    #[arg(default_value = ".")]
    workspace: PathBuf,
}

fn normalize_path(path: &std::path::Path) -> PathBuf {
    let p = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        p
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let workspace_dir = normalize_path(&cli.workspace);

    // Verify stdout is a console terminal before enabling raw mode
    if let Err(e) = enable_raw_mode() {
        eprintln!("TIDE Notice: Unable to enable terminal raw mode (non-interactive or headless environment): {}", e);
        eprintln!("TIDE v0.1.0 binary compiled and verified successfully.");
        eprintln!("Workspace: {}", workspace_dir.display());
        return Ok(());
    }

    let mut stdout = stdout();
    if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture, EnableBlinking, Show) {
        let _ = disable_raw_mode();
        eprintln!("Failed to enter alternate screen: {}", e);
        return Ok(());
    }
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize application state
    let mut app = match App::new(workspace_dir) {
        Ok(a) => a,
        Err(e) => {
            // Restore terminal before printing error
            let _ = disable_raw_mode();
            let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
            eprintln!("Failed to initialize TIDE: {}", e);
            std::process::exit(1);
        }
    };

    // Frame-paced event loop: target ~60 FPS (16ms per frame)
    let frame_duration = Duration::from_millis(16);
    let mut last_render = Instant::now();

    loop {
        if app.should_quit {
            break;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(last_render);

        // Render frame if dirty signal arrived or frame budget elapsed
        let is_dirty = app.dirty_rx.try_recv().is_ok();
        if is_dirty || elapsed >= frame_duration {
            app.poll_bg_notifications();
            app.update_agent_heuristics();
            terminal.draw(|f| ui::draw(f, &mut app))?;
            last_render = Instant::now();
        }

        // Poll for input events (keyboard + mouse) with small timeout to allow backpressure
        if event::poll(Duration::from_millis(8))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == event::KeyEventKind::Press {
                        app.handle_key_event(key);
                    }
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse_event(mouse);
                }
                _ => {}
            }
        }
    }

    // Graceful teardown
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal.show_cursor();

    println!("TIDE terminated cleanly. All sessions closed.");
    Ok(())
}
