use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use zbus::{zvariant::Value, Message};

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
	pub app_name: String,
	pub title: String,
	pub body: String,
}

type MessageBody<'a> =
	(String, u32, &'a str, String, String, Vec<&'a str>, HashMap<&'a str, Value<'a>>, i32);

impl TryFrom<Arc<Message>> for Notification {
	type Error = zbus::Error;

	fn try_from(msg: Arc<Message>) -> Result<Self, Self::Error> {
		let (app_name, _, _, title, body, ..) = msg.body::<MessageBody>()?;

		Ok(Notification { app_name, title, body })
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
