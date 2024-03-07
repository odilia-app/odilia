use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use zbus::{zvariant::Value, Message};

use crate::action::Action;
use crate::urgency::Urgency;
use itertools::Itertools;

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
	pub app_name: String,
	pub title: String,
	pub body: String,
	pub urgency: Urgency,
	pub actions: Vec<Action>,
}

type MessageBody<'a> =
	(String, u32, &'a str, String, String, Vec<&'a str>, HashMap<&'a str, Value<'a>>, i32);

impl TryFrom<Arc<Message>> for Notification {
	type Error = zbus::Error;

	fn try_from(msg: Arc<Message>) -> Result<Self, Self::Error> {
		let mb: MessageBody = msg.body()?;
		let (app_name, _, _, title, body, actions, mut options, _) = mb;
		let actions = actions
			.iter()
			.tuples()
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
	use zbus::names::UniqueName;

	use super::*;
	#[test]
	fn correctly_formatted_message_leads_to_a_correct_notification() -> Result<(), zbus::Error>
	{
		// Simulate a method call to the org.freedesktop.notifications interface
		let message = Message::method(
			Some(":0.1"), //I can't pass none here, because of type needed errors, so passing dummy values for now
			Some(":0.3"), //same here
			"/org/freedesktop/notifications",
			Some("org.freedesktop.notifications"),
			"notify",
			&(
				"ExampleApp",
				0u32,
				"summary",
				"Test Title",
				"Test Body",
				Vec::<&str>::new(),
				HashMap::<&str, Value>::new(),
				0,
			),
		)?;

		//make this into an arc, to use the try_from implementation used in the wild
		let message = Arc::new(message);
		// Convert the Message into a Notification
		let notification = Notification::try_from(message)?;

		// Assert that the conversion was successful and the fields are as expected
		assert_eq!(notification.app_name, "ExampleApp");
		assert_eq!(notification.title, "Test Title");
		assert_eq!(notification.body, "Test Body");

		Ok(())
	}
}
