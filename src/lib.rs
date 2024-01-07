use futures::{Stream, StreamExt};
use std::{collections::HashMap, error::Error};
use tracing::{info, instrument};

use serde::{Deserialize, Serialize};

use zbus::{zvariant::Value, Connection, MessageStream};

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
#[instrument]
pub async fn listen_to_dbus_notifications(
) -> impl Stream<Item = Result<Notification, Box<dyn Error + Send + Sync + 'static>>> {
    info!("initializing dbus connection");
    let connection = Connection::session().await.unwrap();
    info!("setting dbus connection to monitor mode");
    connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus.Monitoring"),
            "BecomeMonitor",
            &(&[] as &[&str], 0u32),
        )
        .await
        .unwrap();
    info!("connection is now in monitor mode");
    info!("listening for notifications");
    MessageStream::from(connection)
        .fuse()
        .filter_map(|message| {
            match message {
                Ok(msg) => {
                    if msg.interface() == Some("org.freedesktop.Notifications".try_into().unwrap())
                        && msg.member() == Some("Notify".try_into().unwrap())
                    {
                        let (app_name, _, _, summary, body, actions, _, _): (
                            String,
                            u32,
                            String,
                            String,
                            String,
                            Vec<String>,
                            HashMap<String, Value>,
                            i32,
                        ) = msg.body().unwrap();
                        info!(
                            app_name = app_name,
                            body = body,
                            "got a notification, adding it to stream"
                        );
                        futures::future::ready(Some(Ok(Notification {
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
                        })))
                    } else {
                        futures::future::ready(None)
                    }
                }
                Err(_) => futures::future::ready(None),
            }
        })
}
