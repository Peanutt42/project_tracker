use project_tracker_core::DatabaseMessage;
use crate::project_tracker::Message;

impl From<DatabaseMessage> for Message {
	fn from(value: DatabaseMessage) -> Self {
		Message::DatabaseMessage(value)
	}
}