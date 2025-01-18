use crate::{
	components::{confirm_cancel_button, confirm_ok_button},
	project_tracker,
	styles::{card_style, SPACING_AMOUNT},
};
use iced::{
	widget::{row, text},
	Element,
};
use iced_aw::card;

#[derive(Clone, Debug)]
pub enum Message {
	Open {
		title: String,
		on_confirmed: Box<project_tracker::Message>,
		custom_ok_label: Option<&'static str>,
		custom_cancel_label: Option<&'static str>,
	},
	Close,
}

impl Message {
	pub fn open(
		title: String,
		on_confirmed: impl Into<project_tracker::Message>,
	) -> project_tracker::Message {
		Self::Open {
			title,
			on_confirmed: Box::new(on_confirmed.into()),
			custom_ok_label: None,
			custom_cancel_label: None,
		}
		.into()
	}
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		project_tracker::Message::ConfirmModalMessage(value)
	}
}

pub struct Modal {
	title: String,
	pub on_confirmed: project_tracker::Message,
	custom_ok_label: Option<&'static str>,
	custom_cancel_label: Option<&'static str>,
}

impl Modal {
	pub fn new(
		title: String,
		on_confirmed: impl Into<project_tracker::Message>,
		custom_ok_label: Option<&'static str>,
		custom_cancel_label: Option<&'static str>,
	) -> Self {
		Self {
			title,
			on_confirmed: on_confirmed.into(),
			custom_ok_label,
			custom_cancel_label,
		}
	}

	pub fn view(&self) -> Element<project_tracker::Message> {
		card(
			text(&self.title),
			row![
				confirm_ok_button(&self.on_confirmed, self.custom_ok_label),
				confirm_cancel_button(self.custom_cancel_label)
			]
			.spacing(SPACING_AMOUNT),
		)
		.max_width(300.0)
		.style(card_style)
		.into()
	}
}
