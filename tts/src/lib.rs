use tokio::{
	sync::{
		mpsc::Receiver,
		broadcast,
	},
	io::{BufReader, BufWriter},
	net::unix::{
		OwnedReadHalf, OwnedWriteHalf
	},
};
use ssip_client::{
	tokio::{
		AsyncClient,
		Request,
	},
	fifo::asynchronous_tokio::Builder,
	ClientName,
};
use pin_utils;
use eyre;

pub async fn create_ssip_client() -> eyre::Result<AsyncClient<BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>>> {
  tracing::debug!("Attempting to register SSIP client odilia:speech");
  let mut ssip_core = Builder::new().build().await?;
	tracing::debug!("Client created. Setting name");
  let client_setup_success = ssip_core.set_client_name(ClientName::new("odilia", "speech")).await?
      .check_client_name_set().await?;
  tracing::debug!("SSIP client registered as odilia:speech");
  return Ok(ssip_core);
}

pub async fn handle_ssip_commands(client: &mut AsyncClient<BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>>, requests: Receiver<Request>, shutdown_tx: &mut broadcast::Receiver<i32>) -> eyre::Result<()> {
	pin_utils::pin_mut!(requests);
	loop {
		tokio::select! {
			request_option = requests.recv() => {
				if request_option.is_none() {
					continue;
				}
				let request = request_option.unwrap();
				tracing::debug!("SSIP command received");
        let response = client
          .send(request).await.unwrap()
          .receive().await.unwrap();
        tracing::debug!("Response from server: {:#?}", response);
				continue;
			}
			_ = shutdown_tx.recv() => {
        tracing::debug!("Attempting to quit SSIP.");
        let response = client
          .send(Request::Quit).await.unwrap()
          .receive().await.unwrap();
        tracing::debug!("Response from server: {:?}", response);
				tracing::debug!("SSIP command interpreter is shut down.");
				break;
			}
		}
	}
	Ok(())
}
