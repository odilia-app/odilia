#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	unsafe_code
)]
#![allow(clippy::multiple_crate_versions)]

use std::{
	io::ErrorKind,
	process::{exit, Command, Stdio},
};

use async_channel::Receiver;
use async_net::unix::UnixStream;
use futures_lite::{io::BufReader, FutureExt};
use futures_util::FutureExt as FatExt;
use odilia_common::errors::OdiliaError;
use smol_cancellation_token::CancellationToken;
use ssip_client_async::{
	async_io::AsyncClient, fifo::asynchronous_async_io::Builder, ClientName, Request,
};

async fn or_cancel<F>(f: F, token: &CancellationToken) -> Result<F::Output, std::io::Error>
where
	F: std::future::Future,
{
	token.cancelled()
		.map(|()| Err(std::io::ErrorKind::TimedOut.into()))
		.or(f.map(Ok))
		.await
}

/// Creates a new async SSIP client which can be sent commends, and can await responses to.
/// # Errors
/// There may be errors when trying to send the initial registration command, or when parsing the response.
#[tracing::instrument(level = "debug", err)]
pub async fn create_ssip_client(
) -> Result<AsyncClient<BufReader<UnixStream>, UnixStream>, OdiliaError> {
	tracing::debug!("Attempting to register SSIP client odilia:speech");
	let mut ssip_core =
		match Builder::default().build().await {
			Ok(ssip) => ssip,
			Err(e) => {
				if e.kind() == ErrorKind::ConnectionRefused {
					tracing::debug!("Speech dispatcher is not active. Attempting to spawn it.");
					Command::new("speech-dispatcher")
						.arg("--spawn")
						.stdin(Stdio::null())
						.stdout(Stdio::null())
						.stderr(Stdio::null())
						.spawn()?;
					tracing::debug!(
						"Attempting to connect to speech-dispatcher again!"
					);
					Builder::default().build().await?
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

/// A handler task for incoming SSIP requests
/// This function will run untill it gets canceled via the cancellation token
///
/// # Errors
///
/// This function will return an error if anything within it fails. It may fail to read a value from the channel, it may fail to run an SSIP command, or fail to parse the response.
/// Errors may also be returned during cleanup via the `cancellation_token` parameter, since shutting down the connection to speech dispatcher can also potentially error.
/// Any of these failures will result in this function exiting with an `Err(_)` variant.
#[tracing::instrument(level = "debug", skip_all, err)]
pub async fn handle_ssip_commands(
	mut client: AsyncClient<BufReader<UnixStream>, UnixStream>,
	mut requests: Receiver<Request>,
	shutdown: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	std::pin::pin!(&mut requests);
	loop {
		let maybe_request = or_cancel(requests.recv(), &shutdown).await;
		let Ok(request_option) = maybe_request else {
			tracing::debug!("Saying goodbye message.");
			client.send(Request::Speak).await?.receive().await?;
			client.send(Request::SendLines(Vec::from(["Quitting Odilia".to_string()])))
				.await?
				.receive()
				.await?;
			tracing::debug!("Attempting to quit SSIP.");
			let response = client.send(Request::Quit).await?.receive().await?;
			tracing::debug!(?response, "Recieved response from server");
			tracing::debug!("SSIP command interpreter shutdown completed");
			break;
		};
		if let Ok(request) = request_option {
			tracing::debug!(?request, "SSIP command received");
			let response = client.send(request).await?.receive().await?;
			tracing::debug!(?response, "Recieved response from server");
		}
	}
	Ok(())
}
