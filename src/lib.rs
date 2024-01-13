use futures::{Stream, StreamExt};
use std::{collections::HashMap, error::Error, ops::Deref};
use tracing::{debug, info, instrument};

use serde::{Deserialize, Serialize};

use zbus::{
    fdo::MonitoringProxy, zvariant::Value, Connection, MatchRule, MessageStream, MessageType,
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

#[instrument]
pub async fn listen_to_dbus_notifications() -> impl Stream<Item = Result<Notification, Box<dyn Error + Send + Sync + 'static>>> {
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
            let (app_name, _, _, summary, body, actions, _, _): RawNotifyMethodSignature =
                message?.body()?;
            info!(
                %app_name,
                %body,
                "got a notification, adding it to stream"
            );
            Ok(Notification {
                app_name,
                title: summary,
                body,
                actions: actions
                    .into_iter()
                    .map(|action| Action {
                        name: action,
                        method: "".into(), // We don't have the method info here
                    })
                    .collect(),
            })
        })
}
