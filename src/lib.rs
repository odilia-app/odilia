use futures::{Stream, StreamExt};
use std::{collections::HashMap, error::Error, ops::Deref, sync::Arc};
use tracing::{debug, info, instrument};

use serde::{Deserialize, Serialize};

use zbus::{
    fdo::MonitoringProxy, zvariant::Value, Connection, MatchRule, Message, MessageStream,
    MessageType,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub actions: Vec<Action>,
}
type RawNotifyMethodSignature<'a> = (
    String,
    u32,
    String,
    String,
    String,
    Vec<String>,
    HashMap<String, Value<'a>>,
    i32,
);
impl TryFrom<Message> for Notification {
    type Error = zbus::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let (app_name, _, _, summary, body, actions, _, _): RawNotifyMethodSignature =
            value.body()?;
        let actions = actions
            .into_iter()
            .map(|action| Action {
                name: action,
                method: "".into(), // We don't have the method info here
            })
            .collect();

        Ok(Notification {
            app_name,
            title: summary,
            body,
            actions,
        })
    }
}
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

    MessageStream::from(monitor.connection())
        //the first signal we get is a name lost signal, because entering monitor mode causes the daemon to make our connection drop all names, even if this one in particular has none.
        //Therefore, we must skip hopefully only one value from the beginning of the stream
        .skip(1)
        .map(|message| {
            let message = Arc::try_unwrap(message?).unwrap(); // Extract the Message from the Arc, I'm not sure whether this will work or not. Todo: try to find a better way of doing this
            let notification = message.try_into()?;
            debug!(?notification, "adding notification to stream");
            Ok(notification)
        })
}
