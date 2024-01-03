use futures::{Stream, StreamExt};
use std::{collections::HashMap, error::Error};
use tracing::{info, instrument};

use serde::{Deserialize, Serialize};
use zbus::{dbus_proxy, zvariant::Value, Connection, SignalStream};

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub method: String,
}

#[derive(Debug)]
pub struct Notification {
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub actions: Vec<Action>,
}
#[dbus_proxy(
    interface = "org.freedesktop.Notifications",
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait FreedesktopNotifications {
    #[dbus_proxy(signal)]
    async fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: HashMap<&str, Value<'s>>,
        expire_timeout: i32,
    );
}

#[instrument]
pub async fn listen_to_dbus_notifications() -> SignalStream<'static> {
    info!("initializing dbus connection");
    let connection = Connection::session().await.unwrap();
    info!("initializing dbus proxy for connection");
    let proxy = FreedesktopNotificationsProxy::builder(&connection)
        .destination("org.freedesktop.Notifications")
        .expect("unable to use the notification thingy")
        .build()
        .await
        .unwrap();
    info!("listening for notifications");
    proxy.receive_signal("Notify").await.unwrap()
}
#[instrument]
pub fn create_stream<'a>(
    signal_stream: SignalStream<'a>,
) -> impl Stream<Item = Result<Notification, Box<dyn Error + Send + Sync + 'static>>> + 'a {
    signal_stream.map(|signal| {
        let (app_name, _, _, summary, body, actions): (
            String,
            u32,
            String,
            String,
            String,
            Vec<String>,
        ) = signal.body().unwrap();
        info!(
            app_name = app_name,
            body = body,
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
