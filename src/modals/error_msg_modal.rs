use crate::{
	components::{copy_to_clipboard_button, error_msg_ok_button},
	project_tracker::Message,
	styles::card_style,
};
use iced::{
	widget::{row, text},
	Element,
};
use iced_aw::card;
use tracing::error;

pub enum ErrorMsgModal {
	Open { error_msg: String },
	Closed,
}

#[derive(Clone, Debug)]
pub enum ErrorMsgModalMessage {
	Open(String),
	Close,
}

impl ErrorMsgModalMessage {
	pub fn open(error_msg: String) -> Message {
		Self::Open(error_msg).into()
	}

	pub fn open_error<E: std::error::Error>(error: E) -> Message {
		Self::Open(format!("{error}")).into()
	}
}

impl From<ErrorMsgModalMessage> for Message {
	fn from(value: ErrorMsgModalMessage) -> Self {
		Message::ErrorMsgModalMessage(value)
	}
}

impl ErrorMsgModal {
	pub fn update(&mut self, message: ErrorMsgModalMessage) {
		match message {
			ErrorMsgModalMessage::Open(error_msg) => {
				error!("{error_msg}");
				*self = ErrorMsgModal::Open { error_msg };
			}
			ErrorMsgModalMessage::Close => {
				*self = ErrorMsgModal::Closed;
			}
		}
	}

	pub fn view(&self) -> Option<Element<Message>> {
		match self {
			ErrorMsgModal::Open { error_msg } => Some(
				card(
					text(error_msg),
					row![
						error_msg_ok_button(),
						copy_to_clipboard_button(error_msg.clone()),
					],
				)
				.max_width(500.0)
				.style(card_style)
				.into(),
			),
			ErrorMsgModal::Closed => None,
		}
	}
}
