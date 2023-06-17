#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	unsafe_code
)]

use ssip_client_async::{
	fifo::asynchronous_tokio::Builder,
	tokio::{AsyncClient},
  Request,
	ClientName,
};
use std::{
	io::ErrorKind,
	process::{exit, Command, Stdio},
	thread, time,
};
use tokio::{
	io::{BufReader, BufWriter},
	net::unix::{OwnedReadHalf, OwnedWriteHalf},
	sync::{broadcast, mpsc::Receiver},
};

/// Creates a new async SSIP client which can be sent commends, and can await responses to.
/// # Errors
/// There may be errors when trying to send the initial registration command, or when parsing the response.
pub async fn create_ssip_client(
) -> eyre::Result<AsyncClient<BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>>> {
	tracing::debug!("Attempting to register SSIP client odilia:speech");
	let mut ssip_core =
		match Builder::new().build().await {
			Ok(ssip) => ssip,
			Err(e) => {
				if e.kind() == ErrorKind::ConnectionRefused {
					tracing::debug!("Speech dispatcher is not active. Attempting to spawn it.");
					Command::new("speech-dispatcher")
              .arg("--spawn")
              .stdin(Stdio::null())
              .stdout(Stdio::null())
              .stderr(Stdio::null())
              .spawn()
              .expect("Error running `speech-dispatcher --spawn`; this is a fatal error.");
					tracing::debug!(
						"Attempting to connect to speech-dispatcher again!"
					);
					thread::sleep(time::Duration::from_millis(500));
					Builder::new().build().await?
				} else {
					tracing::debug!("Speech dispatcher could not be started.");
					exit(1);
				}
			}
		};
	tracing::debug!("Client created. Setting name");
	ssip_core
		.set_client_name(ClientName::new("odilia", "speech"))
		.await?
		.check_client_name_set()
		.await?;
	tracing::debug!("SSIP client registered as odilia:speech");
	Ok(ssip_core)
}

/// A handler task for incomming SSIP requests.
/// This function will run until it receives a requrst via the `shutdown_tx`'s sender half.
///
/// # Errors
///
/// This function will return an error if anything within it fails. It may fail to read a value from the channel, it may fail to run an SSIP command, or fail to parse the response.
/// Errors may also be returned during cleanup via the `shutdown_tx` parameter, since shutting down the connection to speech dispatcher can also potentially error.
/// Any of these failures will result in this function exiting with an `Err(_)` variant.
pub async fn handle_ssip_commands(
	client: &mut AsyncClient<BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>>,
	requests: Receiver<Request>,
	shutdown_tx: &mut broadcast::Receiver<i32>,
) -> eyre::Result<()> {
	tokio::pin!(requests);
	loop {
		tokio::select! {
				      request_option = requests.recv() => {
					      if let Some(request) = request_option {
		  tracing::debug!("SSIP command received");
		  let response = client
		    .send(request).await?
		    .receive().await?;
		  tracing::debug!("Response from server: {:#?}", response);
		}
				      }
				      _ = shutdown_tx.recv() => {
		      tracing::debug!("Saying goodbye message.");
		      client
			      .send(Request::Speak).await?
			      .receive().await?;
		      client
			      .send(Request::SendLines(Vec::from(["Quitting Odilia".to_string()]))).await?
			      .receive().await?;
		      tracing::debug!("Attempting to quit SSIP.");
		      let response = client
			.send(Request::Quit).await?
			.receive().await?;
		      tracing::debug!("Response from server: {:?}", response);
					      tracing::debug!("SSIP command interpreter is shut down.");
					      break;
				      }
			      }
	}
	Ok(())
}
