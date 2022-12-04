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

async fn handle_ssip_commands(client: &mut AsyncClient<BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>>, requests: Receiver<Request>, shutdown_tx: &mut broadcast::Receiver<i32>) {
	pin_utils::pin_mut!(requests);
	loop {
		tokio::select! {
			request = requests.recv() => {
				tracing::debug!("SSIP command received");
				continue;
			}
			_ = shutdown_tx.recv() => {
				tracing::debug!("SSIP command interpreter is shut down.");
				break;
			}
		}
	}
}
