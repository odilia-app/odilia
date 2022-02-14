mod device;
use device::Device;

use std::io;

use evdev::InputEvent;

use tokio::{fs, sync::mpsc};

/// The size of the buffer of [`evdev::InputEvent`]s. Sending more events than this at once will
/// currently stop any more events being sent until at least one is processed.
///
/// This applies backpressure to the event stream. However, this may cause issues since Odilia
/// grabs all keyboard input. If this turns out to be the case, we may need to use an [unbounded
/// channel][tokio::sync::mpsc::unbounded_channel] instead.
pub const MAX_INPUT_EVENTS: usize = 1024;

pub struct InputManager {
    devices: Vec<Device>,
    rx: mpsc::Receiver<InputEvent>,
}

impl InputManager {
/// Initialises the input subsystem, spawning tasks to handle input events.
pub async fn events() -> io::Result<Self> {
    let (tx, rx) = mpsc::channel(MAX_INPUT_EVENTS);
    let devices = open_devices(&tx).await?;
        Ok(Self { devices, rx })
}

#[inline]
pub fn rx_mut(&mut self) -> &mut mpsc::Receiver<InputEvent> {
    &mut self.rx
}
}

    /// Enumerates input devices in `/dev/input`, spawning tasks to handl their
    /// [`EventStream`][evdev::EventStream]s.
    ///
    /// If an error occurs when reading the `/dev/input` directory, the error is returned. If an
    /// error occurs when opening the device, the error is logged and the device is skipped.
async fn open_devices(tx: &mpsc::Sender<InputEvent>) -> io::Result<Vec<Device>> {
        tracing::debug!("Populating input devices from /dev/input");

        let mut devices = Vec::new();
        let mut dir = fs::read_dir("/dev/input").await?;

        loop {
            // Get next directory entry
            let entry = match dir.next_entry().await {
                Ok(Some(entry)) => entry,
                Ok(None) => break, // No more entries
                Err(e) => {
                    tracing::error!(error = %e, "Failed to read directory entry");
                    continue;
                }
            };
            let name_os = entry.file_name();
            // Try to convert to UTF8 (will likely always succeed unless the locale is set
            // to something uncommon)
            match name_os.to_str() {
                Some(name) if name.starts_with("event") => (), // Positive match
                _ => continue, // We're only interested in /dev/event*
            }
            let path = entry.path();
            // Skip directories (I.E. by-id and by-path)
            match entry.file_type().await {
                Ok(t) if !t.is_dir() => (), // Positive match
                Ok(_) => continue,          // Directory, skip it
                Err(e) => {
                    tracing::error!(path = %entry.path().display(), error = %e, "Could not get file type of directory entry");
                    continue;
                }
            }
            // Open the device
            tracing::debug!(path = %path.display(), "Opening input device");
            let dev = match evdev::Device::open(&path) {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!(path = %path.display(), error = %e, "Could not open input device");
                    continue;
                }
            };
            // Convert to an async stream of events
            let stream = match dev.into_event_stream() {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(path = %path.display(), error = %e, "Could not create event stream from input device");
                    continue;
                }
            };
            devices.push(Device::new(path, stream, tx.clone()));
        }
        Ok(devices)
    }
