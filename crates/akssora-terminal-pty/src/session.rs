use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::event::PtyEvent;

// INFO: Manages child process lifecycle and bidirectional communication through native OS pseudo-terminals.
pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    output_rx: mpsc::Receiver<Vec<u8>>,
}

#[cfg(unix)]
fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
}

#[cfg(windows)]
fn default_shell() -> String {
    "cmd.exe".to_string()
}

impl PtySession {
    pub fn spawn(rows: u16, cols: u16) -> anyhow::Result<Self> {
        Self::spawn_in(rows, cols, None)
    }

    pub fn spawn_in(rows: u16, cols: u16, cwd: Option<&std::path::Path>) -> anyhow::Result<Self> {
        let mut cmd = CommandBuilder::new(default_shell());
        cmd.env("TERM", "xterm-256color");
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }
        Self::spawn_with(cmd, rows, cols)
    }

    pub fn spawn_command(command: &str, rows: u16, cols: u16) -> anyhow::Result<Self> {
        let mut cmd = CommandBuilder::new(default_shell());
        cmd.env("TERM", "xterm-256color");
        #[cfg(unix)]
        cmd.args(["-c", command]);
        #[cfg(windows)]
        cmd.args(["/C", command]);
        Self::spawn_with(cmd, rows, cols)
    }

    pub fn spawn_with(cmd: CommandBuilder, rows: u16, cols: u16) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let writer = pair.master.take_writer()?;
        let mut reader = pair.master.try_clone_reader()?;

        // INFO: Bounded sync channel (1024 chunks of 4KB = 4MB buffer) provides backpressure under massive output streams like `yes`
        let (tx, output_rx) = mpsc::sync_channel(1024);
        thread::Builder::new()
            .name("akssora-pty-reader".to_string())
            .spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if tx.send(buf[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            })?;

        Ok(Self {
            master: pair.master,
            writer,
            child,
            output_rx,
        })
    }

    pub fn write_input(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    pub fn resize(&self, rows: u16, cols: u16) -> anyhow::Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    pub fn poll(&self) -> Option<PtyEvent> {
        match self.output_rx.try_recv() {
            Ok(data) => Some(PtyEvent::Data(data)),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(PtyEvent::Closed),
        }
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }

    pub fn process_id(&self) -> Option<u32> {
        self.child.process_id()
    }
}
