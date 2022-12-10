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
	Request,
	tokio::AsyncClient,
};
use pin_utils;
use eyre;

pub async fn create_ssip_client() -> eyre::Result<mut AsyncClient<BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>>> {
  tracing::debug!("Attempting to register SSIP client odilia:speech");
  let mut ssip_core = Builder::new().build().await?;
  let client_setup_success = ssip_core.set_client_name(ClientName::new("odilia", "speech")).await?
      .check_client_name_set().await?;
  tracing::debug!("SSIP client registered as odilia:speech");
  return Ok(ssip_core);
}

async fn handle_ssip_commands(client: &mut AsyncClient<BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>>, requests: Receiver<Request>, shutdown_tx: &mut broadcast::Receiver<i32>) {
	pin_utils::pin_mut!(requests);
	loop {
		tokio::select! {
			request = requests.recv() => {
				tracing::debug!("SSIP command received");
        let response = client
          .send(request).await.unwrap()
          .response().await.unwrap();
        tracing::debug!("Response from server: {:#?}", response);
				continue;
			}
			_ = shutdown_tx.recv() => {
				tracing::debug!("SSIP command interpreter is shut down.");
				break;
			}
		}
	}
}
