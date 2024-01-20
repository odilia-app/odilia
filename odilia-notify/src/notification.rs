use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use zbus::{zvariant::Value, Message};

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub app_name: String,
    pub title: String,
    pub body: String,
}

type MessageBody<'a> = (
    String,
    u32,
    &'a str,
    String,
    String,
    Vec<&'a str>,
    HashMap<&'a str, Value<'a>>,
    i32,
);

impl TryFrom<Message> for Notification {
    type Error = zbus::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let (app_name, _, _, title, body, ..) = value.body::<MessageBody>()?;

        Ok(Notification {
            app_name,
            title,
            body,
        })
    }
}
