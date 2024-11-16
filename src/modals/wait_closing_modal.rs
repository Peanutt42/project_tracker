use crate::{
	components::{force_close_anyways_button, loading_screen, SMALL_LOADING_SPINNER_SIZE}, project_tracker::Message, styles::card_style
};
use iced::{
	alignment::Horizontal, widget::{container, row, text}, window, Element, Length::Fill, Task
};
use iced_aw::card;

#[derive(Clone, Debug)]
pub enum WaitClosingModalMessage {
	Open {
		waiting_reason: &'static str,
	},
	Close,
	ForceCloseAnyways,
}

impl From<WaitClosingModalMessage> for Message {
	fn from(value: WaitClosingModalMessage) -> Self {
		Message::WaitClosingModalMessage(value)
	}
}

pub enum WaitClosingModal {
	Opened {
		waiting_reason: &'static str,
	},
	Closed,
}

impl WaitClosingModal {
	pub fn update(&mut self, message: WaitClosingModalMessage) -> Task<WaitClosingModalMessage> {
		match message {
			WaitClosingModalMessage::Open { waiting_reason } => {
				*self = WaitClosingModal::Opened { waiting_reason };
				Task::none()
			}
			WaitClosingModalMessage::Close => {
				*self = WaitClosingModal::Closed;
				Task::none()
			}
			WaitClosingModalMessage::ForceCloseAnyways => {
				window::get_latest().and_then(window::close)
			}
		}
	}

	pub fn view(&self) -> Option<Element<WaitClosingModalMessage>> {
		match self {
			WaitClosingModal::Closed => None,
			WaitClosingModal::Opened { waiting_reason } => Some(
				card(
					text(format!("{waiting_reason}...")),
					row![
						loading_screen(SMALL_LOADING_SPINNER_SIZE),

						container(force_close_anyways_button())
							.width(Fill)
							.align_x(Horizontal::Right),
					],
				)
				.max_width(300.0)
				.style(card_style)
				.into(),
			),
		}
	}
}