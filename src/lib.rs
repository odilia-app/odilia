use futures::{Stream, StreamExt};
use std::{error::Error, ops::Deref, sync::Arc};
use tracing::{debug, info, instrument};

use zbus::{fdo::MonitoringProxy, Connection, MatchRule, MessageStream, MessageType};
mod notification;
use notification::Notification;

#[instrument]
pub async fn listen_to_dbus_notifications(
) -> impl Stream<Item = Result<Notification, Box<dyn Error + Send + Sync + 'static>>> {
    info!("initializing dbus connection");
    let connection = Connection::session().await.unwrap();
    info!("setting dbus connection to monitor mode");
    let monitor = MonitoringProxy::builder(&connection)
        .destination("org.freedesktop.DBus")
        .unwrap()
        .interface("org.freedesktop.DBus.Monitoring")
        .unwrap()
        .path("/org/freedesktop/DBus")
        .unwrap()
        .build()
        .await
        .unwrap();
    info!("connection is now in monitor mode");
    debug!("creating notifications filtering rule");
    let notify_rule = MatchRule::builder()
        .interface("org.freedesktop.Notifications")
        .unwrap()
        .path("/org/freedesktop/Notifications")
        .unwrap()
        .msg_type(MessageType::MethodCall)
        .member("Notify")
        .unwrap()
        .build();

    debug!(?notify_rule, "finished generating rule");
    info!("listening for notifications");
    let notify_rule = notify_rule.to_string();
    monitor
        .become_monitor(&[notify_rule.deref()], 0)
        .await
        .unwrap();

    MessageStream::from(monitor.connection()).map(|message| {
        let message = Arc::try_unwrap(message?).unwrap(); // Extract the Message from the Arc, I'm not sure whether this will work or not. Todo: try to find a better way of doing this
        let notification = message.try_into()?;
        debug!(?notification, "adding notification to stream");
        Ok(notification)
    })
}
