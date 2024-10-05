use crate::{
	components::{confirm_cancel_button, confirm_ok_button},
	project_tracker::Message,
	styles::{card_style, SPACING_AMOUNT},
};
use iced::{
	widget::{row, text},
	Element,
};
use iced_aw::card;

#[derive(Clone, Debug)]
pub enum ConfirmModalMessage {
	Open {
		title: String,
		on_confirmed: Box<Message>,
	},
	Close,
}

impl ConfirmModalMessage {
	pub fn open(title: String, on_confirmed: impl Into<Message>) -> Message {
		Self::Open {
			title,
			on_confirmed: Box::new(on_confirmed.into()),
		}
		.into()
	}
}

impl From<ConfirmModalMessage> for Message {
	fn from(value: ConfirmModalMessage) -> Self {
		Message::ConfirmModalMessage(value)
	}
}

pub enum ConfirmModal {
	Opened {
		title: String,
		on_confirmed: Message,
	},
	Closed,
}

impl ConfirmModal {
	pub fn update(&mut self, message: ConfirmModalMessage) {
		match message {
			ConfirmModalMessage::Open {
				title,
				on_confirmed,
			} => {
				*self = ConfirmModal::Opened {
					title,
					on_confirmed: *on_confirmed,
				};
			}
			ConfirmModalMessage::Close => {
				*self = ConfirmModal::Closed;
			}
		}
	}

	pub fn view(&self) -> Option<Element<Message>> {
		match self {
			ConfirmModal::Closed => None,
			ConfirmModal::Opened {
				title,
				on_confirmed,
			} => Some(
				card(
					text(title),
					row![confirm_ok_button(on_confirmed), confirm_cancel_button(),]
						.spacing(SPACING_AMOUNT),
				)
				.max_width(300.0)
				.style(card_style)
				.into(),
			),
		}
	}
}