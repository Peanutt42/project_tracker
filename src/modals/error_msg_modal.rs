use crate::{
	components::{copy_to_clipboard_button, error_msg_ok_button},
	project_tracker,
	styles::card_style,
};
use iced::{
	widget::{row, text},
	Element,
};
use iced_aw::card;
use tracing::error;

pub enum Modal {
	Open { error_msg: String },
	Closed,
}

#[derive(Clone, Debug)]
pub enum Message {
	Open(String),
	Close,
}

impl Message {
	pub fn open(error_msg: String) -> project_tracker::Message {
		Self::Open(error_msg).into()
	}

	pub fn open_error<E: std::error::Error>(error: E) -> project_tracker::Message {
		Self::Open(format!("{error}")).into()
	}
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		project_tracker::Message::ErrorMsgModalMessage(value)
	}
}

impl Modal {
	pub fn update(&mut self, message: Message) {
		match message {
			Message::Open(error_msg) => {
				error!("{error_msg}");
				*self = Modal::Open { error_msg };
			}
			Message::Close => {
				*self = Modal::Closed;
			}
		}
	}

	pub fn view(&self) -> Option<Element<project_tracker::Message>> {
		match self {
			Modal::Open { error_msg } => Some(
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
			Modal::Closed => None,
		}
	}
}
