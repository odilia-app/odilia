use futures::TryStreamExt;
use odilia_notify::*;
use std::{collections::HashMap, error::Error, time::Duration};
use zbus::{dbus_proxy, zvariant::Value, Connection};

#[dbus_proxy(
    interface = "org.freedesktop.Notifications",
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait FreedesktopNotifications {
    #[allow(clippy::too_many_arguments)]
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
async fn test_listen_to_dbus_notifications() -> Result<(), Box<dyn Error>> {
    // Create a new connection
    let connection = Connection::session().await.unwrap();
    // Create a proxy for the org.freedesktop.Notifications interface
    let proxy = FreedesktopNotificationsProxy::builder(&connection)
        .destination("org.freedesktop.Notifications")
        .unwrap()
        .build()
        .await?;

    // Spawn a new task to listen for notifications
    let listener_task = tokio::spawn(async move {
        let mut stream=listen_to_dbus_notifications().await;
        while let Some(notification) = stream.try_next().await.unwrap() {
            assert_eq!(notification.app_name, "test_app");
            assert_eq!(notification.title, "Test Summary");
        }
        Ok::<(), Box<dyn Error + Send>>(())
    });
    // Delay sending the notification
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Send a Notification to see if it's correctly recieved on the other side
    proxy
        .notify(
            "test_app",
            0,
            "",
            "Test Summary",
            "Test Body",
            vec![],
            HashMap::new(),
            5000)
        .await?;
    // Await the listener task
    listener_task.await?.unwrap();
    Ok(())
}
