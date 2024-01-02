use futures::StreamExt;

use futures::Stream;
use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use zbus::dbus_proxy;
use zbus::zvariant::Value;
use zbus::Connection;
use zbus::SignalStream;

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
#[dbus_proxy(interface = "org.freedesktop.Notifications", default_service = "org.freedesktop.Notifications", default_path = "/org/freedesktop/Notifications")]
trait FreedesktopNotifications {
    #[dbus_proxy(signal)]
    fn notify(
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

pub async fn listen_to_dbus_notifications() -> SignalStream<'static> {
    let connection = Connection::session().await.unwrap();
    let proxy = FreedesktopNotificationsProxy::builder(&connection)
        .destination("org.freedesktop.Notifications")
        .expect("unable to use the notification thingy")
        .build()
        .await
        .unwrap();
    proxy.receive_signal("Notify").await.unwrap()
}
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
        Ok(Notification {
            app_name: app_name.into(),
            title: summary.into(),
            body: body.into(),
            actions: actions
                .into_iter()
                .map(|action| Action {
                    name: action.into(),
                    method: "".into(), // We don't have the method info here
                })
                .collect(),
        })
    })
}
