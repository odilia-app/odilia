use futures::{Stream, StreamExt};
use std::{ops::Deref, sync::Arc};
use tracing::{debug, info, instrument};

use zbus::{fdo::MonitoringProxy, Connection, MatchRule, MessageStream, MessageType};
mod notification;
use notification::Notification;
mod error;
use error::NotifyError;
#[instrument]
pub async fn listen_to_dbus_notifications() -> Result<impl Stream<Item = Notification>, NotifyError>
{
	info!("initializing dbus connection");
	let connection = Connection::session().await?;
	info!("setting dbus connection to monitor mode");
	let monitor = MonitoringProxy::builder(&connection)
		.destination("org.freedesktop.DBus")?
		.interface("org.freedesktop.DBus.Monitoring")?
		.path("/org/freedesktop/DBus")?
		.build()
		.await?;
	info!("connection is now in monitor mode");
	debug!("creating notifications filtering rule");
	let notify_rule = MatchRule::builder()
		.interface("org.freedesktop.Notifications")?
		.path("/org/freedesktop/Notifications")?
		.msg_type(MessageType::MethodCall)
		.member("Notify")?
		.build();
	debug!(?notify_rule, "finished generating rule");
	info!("listening for notifications");
	let notify_rule = notify_rule.to_string();
	monitor.become_monitor(&[notify_rule.deref()], 0).await?;

	let stream = MessageStream::from(monitor.connection()).filter_map(move |message| async {
		let message = Arc::into_inner(message.ok()?)?; // Extract the Message from the Arc, I'm not sure whether this will work or not. Todo: try to find a better way of doing this
		let notification = message.try_into().ok()?;
		debug!(?notification, "adding notification to stream");
		Some(notification)
	});
	//pinn the stream on the heap, because it's otherwise unusable. Warning: this inccurs additional memory allocations and is not exactly pretty, so alternative solutions should be found
	Ok(Box::pin(stream))
}
