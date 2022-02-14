use std::{ffi::{OsStr, OsString}, io, path::PathBuf};

use tokio::{sync::mpsc, task::JoinHandle};

pub struct Device {
    task: JoinHandle<()>,
    name: OsString,
}

impl Device {
    pub fn new(
        path: PathBuf,
        tx: mpsc::Sender<evdev::InputEvent>,
    ) -> io::Result<Self> {
            tracing::debug!(path = %path.display(), "Opening input device");

            let dev = match evdev::Device::open(&path) {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!(path = %path.display(), error = %e, "Could not open input device");
                    return Err(e);
                }
            };
            // Convert to an async stream of events
            let mut stream = match dev.into_event_stream() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(path = %path.display(), error = %e, "Could not create event stream from input device");
                    return Err(e);
                }
            };
        let name = path
            .file_name()
            .expect("Input device should have a file name")
            .to_os_string();
        let task = tokio::spawn(async move {
            loop {
                match stream.next_event().await {
                    Ok(event) => {
                        if let Err(e) = tx.send(event).await {
                            tracing::warn!(error = %e, "Input event could not be processed");
                        }
                    }
                    Err(e) => {
                        tracing::error!(path = %path.display(), error = %e, "Failed to read from input device")
                    }
                }
            }
        });
        Ok(Self { name, task })
    }

    pub fn name(&self) -> &OsStr {
        &self.name
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // Stop processing events from this device on drop
        self.task.abort();
    }
}
