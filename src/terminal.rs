//! PTY management and VT100 terminal emulation for TIDE.
//!
//! Spawns real interactive shell or agent processes using Windows ConPTY (via portable-pty),
//! binds them to a Windows Job Object for clean process-tree teardown, and maintains
//! an in-memory vt100 terminal grid for responsive, frame-paced rendering.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use crossbeam_channel::Sender;

use crate::job_object::JobObject;

#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};
#[cfg(windows)]
use windows_sys::Win32::Foundation::CloseHandle;

pub struct TerminalSession {
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    pub parser: Arc<Mutex<vt100::Parser>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    _job: JobObject,
    pub is_alive: Arc<Mutex<bool>>,
    pub last_activity: Arc<Mutex<std::time::Instant>>,
}

/// Resolves an executable name or agent command across PATH, user profile, and shell wrappers.
pub fn resolve_command(command: &str, args: &[&str]) -> (String, Vec<String>) {
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    // Direct existing file check
    let p = std::path::Path::new(command);
    if p.is_file() {
        return (command.to_string(), args_vec);
    }

    #[cfg(windows)]
    {
        let userprofile = std::env::var("USERPROFILE").unwrap_or_default();
        let home = std::path::Path::new(&userprofile);

        // Special handling for coding agents on Windows
        if command.eq_ignore_ascii_case("claude") {
            let claude_exe = home.join(".local").join("bin").join("claude.exe");
            if claude_exe.is_file() {
                return (claude_exe.to_string_lossy().to_string(), args_vec);
            }
        }

        if command.eq_ignore_ascii_case("agy") {
            let agy_exe = home.join("AppData").join("Local").join("agy").join("bin").join("agy.exe");
            if agy_exe.is_file() {
                return (agy_exe.to_string_lossy().to_string(), args_vec);
            }
        }

        // Search PATH directories for command, command.exe, command.cmd, command.bat
        if let Ok(path_var) = std::env::var("PATH") {
            for dir_str in path_var.split(';') {
                let trimmed = dir_str.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let dir = std::path::Path::new(trimmed);
                let search_dir = if dir.is_file() {
                    dir.parent().unwrap_or(dir)
                } else {
                    dir
                };

                let candidates = [
                    search_dir.join(command),
                    search_dir.join(format!("{}.exe", command)),
                    search_dir.join(format!("{}.cmd", command)),
                    search_dir.join(format!("{}.bat", command)),
                ];

                for candidate in candidates {
                    if candidate.is_file() {
                        let path_str = candidate.to_string_lossy().to_string();
                        if path_str.ends_with(".cmd") || path_str.ends_with(".bat") {
                            let mut full_args = vec!["/c".to_string(), path_str];
                            full_args.extend(args_vec);
                            return ("cmd.exe".to_string(), full_args);
                        }
                        return (path_str, args_vec);
                    }
                }
            }
        }

        // Common install locations
        let fallback_candidates = [
            home.join(".local").join("bin").join(format!("{}.exe", command)),
            home.join("AppData").join("Local").join(command).join("bin").join(format!("{}.exe", command)),
            home.join("AppData").join("Roaming").join("npm").join(format!("{}.cmd", command)),
        ];

        for cand in fallback_candidates {
            if cand.is_file() {
                let path_str = cand.to_string_lossy().to_string();
                if path_str.ends_with(".cmd") || path_str.ends_with(".bat") {
                    let mut full_args = vec!["/c".to_string(), path_str];
                    full_args.extend(args_vec);
                    return ("cmd.exe".to_string(), full_args);
                }
                return (path_str, args_vec);
            }
        }
    }

    (command.to_string(), args_vec)
}

impl TerminalSession {
    pub fn spawn(
        id: &str,
        title: &str,
        command: &str,
        args: &[&str],
        cwd: Option<&str>,
        rows: u16,
        cols: u16,
        dirty_tx: Sender<()>,
    ) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let job = JobObject::new()?;

        let (resolved_cmd, resolved_args) = resolve_command(command, args);
        let mut cmd = CommandBuilder::new(&resolved_cmd);
        for arg in &resolved_args {
            cmd.arg(arg);
        }
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }

        let child = pair.slave.spawn_command(cmd)?;

        // Attach child process tree to the Windows Job Object
        #[cfg(windows)]
        if let Some(pid) = child.process_id() {
            unsafe {
                let proc_handle = OpenProcess(
                    PROCESS_SET_QUOTA | PROCESS_TERMINATE,
                    0,
                    pid,
                );
                if !proc_handle.is_null() {
                    let _ = job.assign_process(proc_handle as _);
                    CloseHandle(proc_handle);
                }
            }
        }

        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 1000)));
        let writer = Arc::new(Mutex::new(pair.master.take_writer()?));
        let mut reader = pair.master.try_clone_reader()?;

        let parser_clone = Arc::clone(&parser);
        let is_alive = Arc::new(Mutex::new(true));
        let is_alive_clone = Arc::clone(&is_alive);
        let last_activity = Arc::new(Mutex::new(std::time::Instant::now()));
        let activity_clone = Arc::clone(&last_activity);

        // Background reader thread: reads PTY bytes into VT100 grid and signals UI loop
        thread::Builder::new()
            .name(format!("pty-reader-{}", id))
            .spawn(move || {
                let mut buf = [0u8; 8192];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Ok(mut p) = parser_clone.lock() {
                                p.process(&buf[..n]);
                            }
                            if let Ok(mut act) = activity_clone.lock() {
                                *act = std::time::Instant::now();
                            }
                            let _ = dirty_tx.try_send(());
                        }
                        Err(_) => break,
                    }
                }
                if let Ok(mut alive) = is_alive_clone.lock() {
                    *alive = false;
                }
                let _ = dirty_tx.try_send(());
            })?;

        let master = Arc::new(Mutex::new(pair.master));

        Ok(Self {
            id: id.to_string(),
            title: title.to_string(),
            parser,
            writer,
            master,
            _job: job,
            is_alive,
            last_activity,
        })
    }

    /// Sends raw input bytes (e.g. keyboard keystrokes) to the child PTY.
    pub fn write_input(&self, input: &[u8]) -> anyhow::Result<()> {
        if let Ok(mut act) = self.last_activity.lock() {
            *act = std::time::Instant::now();
        }
        if let Ok(mut w) = self.writer.lock() {
            w.write_all(input)?;
            w.flush()?;
        }
        Ok(())
    }

    /// Resizes the PTY and virtual screen grid when the UI pane changes dimensions.
    pub fn resize(&self, rows: u16, cols: u16) {
        if let Ok(master) = self.master.lock() {
            let _ = master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
        if let Ok(mut parser) = self.parser.lock() {
            parser.set_size(rows, cols);
        }
    }
}
