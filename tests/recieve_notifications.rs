use futures::StreamExt;
use odilia_notify::*;
use std::{collections::HashMap, error::Error};
use zbus::{dbus_proxy, zvariant::Value, Connection};

#[dbus_proxy(
    interface = "org.freedesktop.Notifications",
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
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
    ) -> Result<(), Box<dyn Error>>;
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

    // Spawn a new task to listen for notifications
    let listener_task = tokio::spawn(async move {
        listen_to_dbus_notifications()
            .await
            .for_each(|result| {
                match result {
                    Ok(notification) => {
                        assert_eq!(notification.app_name, "test_app");
                        assert_eq!(notification.title, "Test Summary");
                        // Add more assertions for the other fields of the notification
                    }
                    Err(_) => {}
                }
                futures::future::ready(())
            })
            .await;
    });

    // Delay sending the notification
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Send a Notify signal
    if let Err(_) = proxy
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
    {
        eprintln!("something went terribly wrong")
    }

    // Await the listener task
    listener_task.await.unwrap();
}
