#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	unsafe_code,
	clippy::print_stdout,
	clippy::print_stderr
)]
#![allow(clippy::multiple_crate_versions)]

mod proxy;

use std::{
	env,
	ops::ControlFlow,
	os::unix::net::SocketAddr,
	path::Path,
	process::{exit, id},
	time::{SystemTime, UNIX_EPOCH},
};

use async_channel::Sender;
use async_fs as fs;
use async_net::unix::{UnixListener, UnixStream};
use futures_lite::{future::or, stream::Stream, AsyncReadExt};
use futures_util::{future::BoxFuture, FutureExt};
use nix::unistd::Uid;
use odilia_common::{errors::OdiliaError, events::ScreenReaderEvent};
use smol_cancellation_token::CancellationToken;
use sysinfo::{ProcessExt, System, SystemExt};

async fn or_cancel<F>(f: F, token: &CancellationToken) -> Result<F::Output, std::io::Error>
where
	F: std::future::Future,
{
	or(token.cancelled().map(|()| Err(std::io::ErrorKind::TimedOut.into())), f.map(Ok)).await
}

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

/// Open a socket to handle Odilia's input events from an input server.
/// This function will exit upon the expiry of the cancellation token passed in.
/// # Errors
/// - This function will return an error type if the same function is already running. This is checked by looking for a file on disk. If the file exists, this program is probably already running.
/// - If there is no way to get access to the directory.
pub async fn setup_input_server() -> Result<UnixListener, OdiliaError> {
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
		let odilias_pid = fs::read_to_string(&pid_file_path).await?;
		tracing::debug!("Previous PID: {}", odilias_pid);

		let mut sys = System::new_all();
		sys.refresh_all();
		for (pid, process) in sys.processes() {
			if pid.to_string() == odilias_pid && process.exe() == env::current_exe()? {
				return Err("Server is already running".into());
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

	let listener = UnixListener::bind(sock_file_path)?;
	tracing::debug!("Listener activated");
	Ok(listener)
}

/// Receives [`odilia_common::events::ScreenReaderEvent`] structs, then creates a stream of futures that _need_ to be awaited/spawned by the caller onto the executor.
/// Normally, the best way to do this is like so:
///
/// ```rust,no_run
/// # use futures_lite::stream::StreamExt;
/// # use smol_cancellation_token::CancellationToken;
/// # use async_channel::bounded;
/// # use async_net::unix::UnixListener;
/// use odilia_input::sr_event_receiver;
/// // use smol::spawn or tokio::spawn
/// # fn spawn<F>(_f: F) {}
/// let listener = UnixListener::bind("/some/path/here")
///     .expect("Valid listener");
/// let (sender, _receiver) = bounded(128);
/// let ct = CancellationToken::new();
/// // For tokio; for async-io based executors, remember to call .detach()
/// let stream = sr_event_receiver(listener, sender, ct)
///     .for_each(|fut| spawn(fut));
/// ```
///
/// If the cancellation token is triggered, this stream will finish.
#[tracing::instrument(skip_all)]
pub fn sr_event_receiver(
	listener: UnixListener,
	event_sender: Sender<ScreenReaderEvent>,
	shutdown: CancellationToken,
) -> impl Stream<Item = BoxFuture<'static, ()>> {
	async_stream::stream! {
	  loop {
	      match sr_event_receiver_inner(&listener, &event_sender, &shutdown).await {
		Ok(box_fut) => yield box_fut,
		Err(ControlFlow::Break(())) => break,
		Err(ControlFlow::Continue(())) => {},
	    }
	  }
	}
}

/// This contains the logic for [`sr_event_receiver`].
/// Since formatting doesn't work inside macros, we compute the result here, and send it back up to
/// be yielded/break the loop.
async fn sr_event_receiver_inner(
	listener: &UnixListener,
	event_sender: &Sender<ScreenReaderEvent>,
	shutdown: &CancellationToken,
) -> Result<BoxFuture<'static, ()>, ControlFlow<()>> {
	let maybe_msg = or_cancel(listener.accept(), shutdown).await;
	let Ok(msg) = maybe_msg else {
		tracing::debug!("Shutting down listening for new input sockets on '{:?}' due to cancellation token", listener.local_addr());
		return Err(ControlFlow::Break(()));
	};
	match msg {
		Ok((socket, address)) => {
			tracing::debug!("Ok from socket");
			return Ok(handle_event(
				socket,
				address,
				event_sender.clone(),
				shutdown.clone(),
			)
			.boxed());
		}
		Err(e) => {
			tracing::error!("accept function failed: {:?}", e);
		}
	}
	Err(ControlFlow::Continue(()))
}

async fn handle_event(
	mut socket: UnixStream,
	address: SocketAddr,
	event_sender: Sender<ScreenReaderEvent>,
	shutdown: CancellationToken,
) {
	loop {
		let mut buf = [0; 4096];
		let maybe_reader = or_cancel(socket.read(&mut buf), &shutdown).await;
		let Ok(reader) = maybe_reader else {
			tracing::debug!("Shutting down listening on input socket at path '{:?}' due to cancellation token", socket.local_addr());
			break;
		};
		let bytes = match reader {
			Ok(0) => {
				tracing::debug!(
					"Socket '{:?}' was disconnected",
					socket.local_addr()
				);
				break;
			}
			Ok(b) => b,
			Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
				continue;
			}
			Err(e) => {
				tracing::error!(error = ?e, "Error reading from socket");
				continue;
			}
		};
		let response = std::str::from_utf8(&buf[..bytes]).expect("Valid UTF-8");
		// if valid screen reader event

		let sre = match serde_json::from_str::<ScreenReaderEvent>(response) {
			Ok(sre) => sre,
			Err(e) => {
				tracing::error!(error = ?e, "Invalid odilia event");
				continue;
			}
		};
		if let Err(e) = event_sender.send(sre).await {
			tracing::error!(error = ?e, "Error sending ScreenReaderEvent over socket");
		} else {
			tracing::debug!("Sent SR event");
		}
		tracing::debug!(?address, response,);
	}
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
