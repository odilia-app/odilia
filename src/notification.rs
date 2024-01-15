use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use zbus::{
    zvariant::{Type, Value},
    Message,
};

use action::Action;

use crate::action;

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub actions: Vec<Action>,
}

// This struct is used to deserialize the body of the `Notify`` method call message.
// The notification spec: https://specifications.freedesktop.org/notification-spec/notification-spec-latest.html
#[derive(Debug, PartialEq, Serialize, Deserialize, Type)]
struct NotifyArgs<'a> {
    // Human readable app_name.
    app_name: String,

    // Replaces an existing notification.
    replace_id: u32,

    // The app_icon is displayed next to the notification.
    // Further reading: https://specifications.freedesktop.org/notification-spec/notification-spec-latest.html#icons-and-images
    app_icon: String,

    // Brief summary of the notification.
    summary: String,

    // Detailed body text.
    body: String,

    // 	Actions as a list of pairs: Action identifier, localized string that will be displayed.
    actions: Vec<String>,

    // 	Hints as a dictionary. See hints section for more details.
    #[serde(borrow)]
    hints: HashMap<String, Value<'a>>,

    // Timeout time in milliseconds
    expire_timeout: i32,
}

impl TryFrom<Message> for Notification {
    type Error = zbus::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let NotifyArgs {
            app_name,
            summary,
            body,
            actions,
            ..
        } = value.body()?;

        let actions = actions
            .chunks(2)
            .filter_map(|pair| {
                // We expect the actions to be a list of pairs as described in the spec.
                // If the list is not a multiple of 2, we ignore the last element.
                let [identifier, localized_description] = pair else {
                    return None;
                };

                Some(Action {
                    identifier: identifier.to_owned(),
                    description: localized_description.to_owned(),
                })
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
