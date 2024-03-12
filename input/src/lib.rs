#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	unsafe_code
)]
#![allow(clippy::multiple_crate_versions)]

use eyre::Context;
use nix::unistd::Uid;
use odilia_common::events::ScreenReaderEvent;
use std::{
	env,
	path::Path,
	process::{exit, id},
	time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::{fs, io::AsyncReadExt, net::UnixListener, sync::mpsc::Sender};
use tokio_util::sync::CancellationToken;

#[tracing::instrument(ret)]
fn get_log_file_name() -> String {
	tracing::info!("getting unix timestamp for current time");
	let time = if let Ok(n) = SystemTime::now().duration_since(UNIX_EPOCH) {
		tracing::info!(timestamp=?n, "success");
		n.as_secs().to_string()
	} else {
		tracing::error!("SystemTime before UnixEpoch? how is that possible?");
		exit(1);
	};
	tracing::info!("searching for xdg environment variables");
	match env::var("XDG_DATA_HOME") {
		Ok(val) => {
			tracing::info!(
                "XDG_DATA_HOME Variable is present, using it's value for default file path."
            );
			format!("{val}/sohks/sohks-{time}.log")
		}
		Err(e) => {
			tracing::warn!(
                "XDG_DATA_HOME Variable is not set, falling back on hardcoded path.\nError: {:#?}",
                e
            );
			let home = env::var("HOME").expect("No $HOME found in environment.");
			format!("{home}/.local/share/sohks/sohks-{time}.log")
		}
	}
}

/// Receives [`odilia_common::events::ScreenReaderEvent`] structs, then sends them over the `event_sender` socket.
/// This function will exit upon the expiry of the cancellation token passed in.
/// # Errors
/// This function will return an error type if the same function is already running.
/// This is checked by looking for a file on disk. If the file exists, this program is probably already running.
/// If there is no way to get access to the directory, then this function will call `exit(1)`; TODO: should probably return a result instead.
#[tracing::instrument(skip_all)]
pub async fn sr_event_receiver(
	event_sender: Sender<ScreenReaderEvent>,
	shutdown: CancellationToken,
) -> eyre::Result<()> {
	let (pid_file_path, sock_file_path) = get_file_paths();
	let log_file_name = get_log_file_name();

	let log_path = Path::new(&log_file_name);
	tracing::debug!("Socket file located at: {:?}", sock_file_path);
	tracing::debug!("creating log directory");
	if let Some(p) = log_path.parent() {
		if !p.exists() {
			if let Err(e) = fs::create_dir_all(p).await {
				tracing::error!("Failed to create log dir: {}", e);
			}
		}
	}

	tracing::debug!("checking for already running program");
	if Path::new(&pid_file_path).exists() {
		tracing::debug!(
			%pid_file_path, "Reading pid file  and checking for running instances"

		);
		let odilias_pid = match fs::read_to_string(&pid_file_path).await {
			Ok(odilias_pid) => odilias_pid,
			Err(e) => {
				tracing::error!(
					"Unable to read {} to check all running instances",
					e
				);
				exit(1);
			}
		};
		tracing::debug!("Previous PID: {}", odilias_pid);

		let mut sys = System::new_all();
		sys.refresh_all();
		for (pid, process) in sys.processes() {
			if pid.to_string() == odilias_pid && process.exe() == env::current_exe()? {
				tracing::error!("Server is already running!");
				exit(1);
			}
		}
	}

	if Path::new(&sock_file_path).exists() {
		tracing::debug!("Sockfile exists, attempting to remove it.");
		match fs::remove_file(&sock_file_path).await {
			Ok(()) => {
				tracing::debug!("Removed old socket file");
			}
			Err(e) => {
				tracing::error!("Error removing the socket file!: {}", e);
				tracing::error!(
					"You can manually remove the socket file: {}",
					sock_file_path
				);
				exit(1);
			}
		};
	}
	tracing::debug!(%pid_file_path, "writing current ID to pid file");

	match fs::write(&pid_file_path, id().to_string()).await {
		Ok(()) => {}
		Err(e) => {
			tracing::error!("Unable to write to {}: {}", pid_file_path, e);
			exit(1);
		}
	}

	let listener = UnixListener::bind(sock_file_path).context("Could not open socket")?;
	tracing::debug!("Listener activated!");
	loop {
		tokio::select! {
			msg = listener.accept() => {
			    match msg {
				Ok((mut socket, address)) => {
				    tracing::debug!("Ok from socket");
				    let mut response = String::new();
				    match socket.read_to_string(&mut response).await {
				      Ok(_) => {},
				      Err(e) => {
					tracing::error!("Error reading from socket {:#?}", e);
				      }
				    }
				    // if valid screen reader event
				    match serde_json::from_str::<ScreenReaderEvent>(&response) {
				      Ok(sre) => {
					if let Err(e) = event_sender.send(sre).await {
					  tracing::error!("Error sending ScreenReaderEvent over socket: {}", e);
		} else {
					  tracing::debug!("Sent SR event");
		}
				      },
				      Err(e) => tracing::debug!("Invalid odilia event. {:#?}", e),
				    }
				    tracing::debug!("Socket: {:?} Address: {:?} Response: {}", socket, address, response);
				},
				Err(e) => tracing::error!("accept function failed: {:?}", e),
			    }
			    continue;
			}
			() = shutdown.cancelled() => {
			    tracing::debug!("Shutting down input socket due to cancellation token");
			    break;
			}
		    }
	}
	Ok(())
}

#[tracing::instrument(ret)]
fn get_file_paths() -> (String, String) {
	match env::var("XDG_RUNTIME_DIR") {
		Ok(val) => {
			tracing::info!(
                "XDG_RUNTIME_DIR Variable is present, using it's value as default file path."
            );

			let pid_file_path = format!("{val}/odilias.pid");
			let sock_file_path = format!("{val}/odilia.sock");

			(pid_file_path, sock_file_path)
		}
		Err(e) => {
			tracing::warn!(error=%e, "XDG_RUNTIME_DIR Variable is not set, falling back to hardcoded path");

			let pid_file_path = format!("/run/user/{}/odilias.pid", Uid::current());
			let sock_file_path = format!("/run/user/{}/odilia.sock", Uid::current());

			(pid_file_path, sock_file_path)
		}
	}
}
