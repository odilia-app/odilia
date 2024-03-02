# odilia notify

a crate which provides notification service to the odilia screenreader

it works by monitoring the connection to the session bus for method calls applications do when trying to send a notification, converts that into a more usable form, then presents it as a stream of simple data objects

then, consumers of this crate, in this case odilia, can pole the stream for items, one item at a time.

The `Notification` struct is an object with all public fields, which would normally be a problem because mutation with nonsensical values could be done, then invalid data would be introduced in the system. In this case though, those objects are only returned, and there's no way to introduce them back anywhere, so mutating them does nothing more than corrupting one's own data