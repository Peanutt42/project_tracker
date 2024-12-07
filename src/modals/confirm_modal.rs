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
		custom_ok_label: Option<&'static str>,
		custom_cancel_label: Option<&'static str>,
	},
	Close,
}

impl ConfirmModalMessage {
	pub fn open(title: String, on_confirmed: impl Into<Message>) -> Message {
		Self::Open {
			title,
			on_confirmed: Box::new(on_confirmed.into()),
			custom_ok_label: None,
			custom_cancel_label: None,
		}
		.into()
	}

	pub fn open_labeled(title: String, on_confirmed: impl Into<Message>, ok_label: &'static str, cancel_label: &'static str) -> Message {
		Self::Open {
			title,
			on_confirmed: Box::new(on_confirmed.into()),
			custom_ok_label: Some(ok_label),
			custom_cancel_label: Some(cancel_label),
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
		custom_ok_label: Option<&'static str>,
		custom_cancel_label: Option<&'static str>,
	},
	Closed,
}

impl ConfirmModal {
	pub fn update(&mut self, message: ConfirmModalMessage) {
		match message {
			ConfirmModalMessage::Open {
				title,
				on_confirmed,
				custom_ok_label,
				custom_cancel_label,
			} => {
				*self = ConfirmModal::Opened {
					title,
					on_confirmed: *on_confirmed,
					custom_ok_label,
					custom_cancel_label,
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
				custom_ok_label,
				custom_cancel_label,
			} => Some(
				card(
					text(title),
					row![
						confirm_ok_button(on_confirmed, *custom_ok_label),
						confirm_cancel_button(*custom_cancel_label)
					]
					.spacing(SPACING_AMOUNT),
				)
				.max_width(300.0)
				.style(card_style)
				.into(),
			),
		}
	}
}