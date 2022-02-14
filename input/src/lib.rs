mod device;
use device::Device;

use std::{io, path::Path};

use evdev::InputEvent;
use futures::stream::StreamExt;
use inotify::{EventMask, Inotify, WatchMask};
use tokio::{fs, sync::mpsc, task::JoinHandle};

/// The size of the buffer of [`evdev::InputEvent`]s. Sending more events than this at once will
/// currently stop any more events being sent until at least one is processed.
///
/// This applies backpressure to the event stream. However, this may cause issues since Odilia
/// grabs all keyboard input. If this turns out to be the case, we may need to use an [unbounded
/// channel][tokio::sync::mpsc::unbounded_channel] instead.
pub const MAX_INPUT_EVENTS: usize = 1024;

pub struct InputManager {
    inotify_task: JoinHandle<()>,
    rx: mpsc::Receiver<InputEvent>,
}

impl InputManager {
/// Initialises the input subsystem, spawning tasks to handle input events.
pub async fn events() -> io::Result<Self> {
    let (tx, rx) = mpsc::channel(MAX_INPUT_EVENTS);
    let mut devices = open_devices(&tx).await?;
    // Watch for inotify events on /dev/input to dynamically add and remove device handlers
    let mut inotify = Inotify::init()?;
    inotify.add_watch("/dev/input", WatchMask::CREATE | WatchMask::DELETE)?;
    // Calculate the optimum buffer size
    let buf_size = inotify::get_buffer_size(Path::new("/dev/input"))?;
    let mut stream = inotify.event_stream(vec![0; buf_size])?;
    // Spawn a task to process inotify events
    // I don't want this here, but if you try to factor it into a function you either have issues
    // with handling errors, or with lifetimes because `stream` refers to `inotify`.
    let inotify_task = tokio::spawn(async move {
        // Process inotify events
        while let Some(res) = stream.next().await {
            let event = match res {
                Ok(ev) => ev,
                Err(e) => {
                    tracing::warn!(error = %e, "Could not read inotify event");
                    continue;
                }
            };
            // We can't do anything if there's no file name
            let name = if let Some(name) = event.name { name } else {
                tracing::warn!(?event, "Received an inotify event with no associated file name, ignoring it");
                continue;
            };
            if event.mask.contains(EventMask::CREATE) {
                let path = Path::new("/dev/input").join(name);
                let device = match Device::new(path, tx.clone()) {
                    Ok(s) => s,
                    _ => continue, // Device::new() already logs errors
                };
            devices.push(device);
            } else if event.mask.contains(EventMask::DELETE) {
                // Find the handler
                if let Some(index) = devices.iter().enumerate().find_map(|(i, d)| if d.name() == name {
                    Some(i)
                } else {
                    None
                }) {
                    devices.swap_remove(index);
                }
            } else {
                tracing::warn!(mask = event.mask.bits(), "Unknown inotify event, ignoring it");
            }
        }
    });

        Ok(Self { inotify_task, rx })
}

#[inline]
pub fn rx_mut(&mut self) -> &mut mpsc::Receiver<InputEvent> {
    &mut self.rx
}
}

impl Drop for InputManager {
    fn drop(&mut self) {
        // Stop watching for inotify events
        self.inotify_task.abort();
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
            // Open the device and spawn the handler task
                let device = match Device::new(path, tx.clone()) {
                    Ok(s) => s,
                    _ => continue, // Device::new() already logs errors
                };
            devices.push(device);
        }
        Ok(devices)
    }
