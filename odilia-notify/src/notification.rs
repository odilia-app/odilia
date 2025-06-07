use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zbus::{zvariant::Value, Message};

use crate::{action::Action, urgency::Urgency};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Notification {
	pub app_name: String,
	pub title: String,
	pub body: String,
	pub urgency: Urgency,
	pub actions: Vec<Action>,
}

type MessageBody<'a> =
	(String, u32, &'a str, String, String, Vec<&'a str>, HashMap<&'a str, Value<'a>>, i32);

impl TryFrom<Message> for Notification {
	type Error = zbus::Error;

	fn try_from(msg: Message) -> Result<Self, Self::Error> {
		let body = msg.body();
		let mb: MessageBody = body.deserialize()?;
		let (app_name, _, _, title, body, actions, mut options, _) = mb;
		// even elements
		let names = actions.iter().step_by(2);
		// odd elements
		let methods = actions.iter().skip(1).step_by(2);
		let actions = names
			.zip(methods)
			.map(|(name, method)| Action {
				name: name.to_string(),
				method: method.to_string(),
			})
			.collect();
		// any error in deserailizing the value (including lack of "urgency" key in options
		// hashmap)  will give it an urgency of Normal
		let urgency = options
			.remove("urgency")
			.and_then(|o| o.try_into().ok())
			.unwrap_or(Urgency::Normal);

		Ok(Notification { app_name, title, body, actions, urgency })
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn correctly_formatted_message_leads_to_a_correct_notification() -> Result<(), zbus::Error>
	{
		let message = Message::method_call("/org/freedesktop/notifications", "notify")?
			.sender(":0.1")?
			.interface("org.freedesktop.notifications")?
			.build(&(
				"ExampleApp",
				0u32,
				"summary",
				"Test Title",
				"Test Body",
				Vec::<&str>::new(),
				HashMap::<&str, Value>::new(),
				0,
			))?;
		// Convert the Message into a Notification
		let notification = Notification::try_from(message)?;

		// Assert that the conversion was successful and the fields are as expected
		assert_eq!(notification.app_name, "ExampleApp");
		assert_eq!(notification.title, "Test Title");
		assert_eq!(notification.body, "Test Body");

		Ok(())
	}
}
