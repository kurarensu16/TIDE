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
    #[allow(dead_code)]
    pub is_alive: Arc<Mutex<bool>>,
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

        let mut cmd = CommandBuilder::new(command);
        for arg in args {
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
        })
    }

    /// Sends raw input bytes (e.g. keyboard keystrokes) to the child PTY.
    pub fn write_input(&self, input: &[u8]) -> anyhow::Result<()> {
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
