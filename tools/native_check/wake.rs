//! Wake the native event loop for fixture commands, independently of UI redraws.
use eframe::egui::Context;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub(super) struct InputWake {
    stop: mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
}

impl InputWake {
    pub(super) fn start(context: Context, output: PathBuf) -> std::io::Result<Self> {
        let (stop, receiver) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("native-input-wake".into())
            .spawn(move || {
                while matches!(
                    receiver.recv_timeout(Duration::from_millis(10)),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ) {
                    let command = sequence(&output.join("command.json"));
                    let reply = sequence(&output.join("reply.json"));
                    if !output.join("startup.json").exists()
                        || command.is_some_and(|command| reply.is_none_or(|reply| command > reply))
                    {
                        // Keep waking until GPU readback acknowledges the command.
                        context.request_repaint();
                    }
                }
            })?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for InputWake {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn sequence(path: &Path) -> Option<u64> {
    let bytes = fs::read(path).ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    value["sequence"].as_u64()
}
