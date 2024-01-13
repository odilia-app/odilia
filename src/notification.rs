use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use zbus::{zvariant::Value, Message};

use action::Action;

use crate::action;

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
