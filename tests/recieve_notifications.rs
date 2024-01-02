use futures::StreamExt;

use std::collections::HashMap;
use odilia_notify::*;
use zbus::{dbus_proxy, Connection, Result, zvariant::Value};

#[dbus_proxy(interface = "org.freedesktop.Notifications", default_service = "org.freedesktop.Notifications", default_path = "/org/freedesktop/Notifications")]
trait FreedesktopNotifications {
    async fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: HashMap<&str, Value<'_>>,
        expire_timeout: i32,
    ) ->Result<()>;
}
#[tokio::test]
async fn test_listen_to_dbus_notifications() {
    // Create a new connection
    let connection = Connection::session().await.unwrap();

    // Create a proxy for the org.freedesktop.Notifications interface
    let proxy = FreedesktopNotificationsProxy::builder(&connection)
        .destination("org.freedesktop.Notifications")
        .unwrap()
        .build()
        .await
        .unwrap();

    // Send a Notify signal
    proxy
        .notify(
            "test_app",
            0,
            "",
            "Test Summary",
            "Test Body",
            vec![],
            HashMap::new(),
            5000,
        )
        .await
        .unwrap();

    // Listen for notifications
    let signal_stream = listen_to_dbus_notifications().await;
    let stream = create_stream(signal_stream);

    // Check whether the notification is received
    let mut stream = stream.boxed();
    while let Some(result) = stream.next().await {
        match result {
            Ok(notification) => {
                assert_eq!(notification.app_name, "test_app");
                assert_eq!(notification.title, "Test Summary");
                // Add more assertions for the other fields of the notification
                break;
            }
            Err(_) => continue,
        }
    }
}
