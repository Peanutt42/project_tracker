use crate::{
	components::copy_to_clipboard_button,
	project_tracker::Message,
	styles::{card_style, dangerous_button_style},
};
use iced::{
	alignment::Horizontal,
	widget::{button, row, text},
	Element,
	Length::Fill,
};
use iced_aw::card;

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
						button(text("Ok").align_x(Horizontal::Center).width(Fill))
							.width(Fill)
							.style(dangerous_button_style)
							.on_press(ErrorMsgModalMessage::Close.into()),
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
