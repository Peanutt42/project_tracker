use crate::{
	components::{force_close_anyways_button, loading_screen, SMALL_LOADING_SPINNER_SIZE},
	project_tracker,
	styles::card_style,
};
use iced::{
	alignment::Horizontal,
	widget::{container, row, text},
	window, Element,
	Length::Fill,
	Task,
};
use iced_aw::card;

#[derive(Clone, Debug)]
pub enum Message {
	Open { waiting_reason: &'static str },
	Close,
	ForceCloseAnyways,
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		project_tracker::Message::WaitClosingModalMessage(value)
	}
}

pub enum Modal {
	Opened { waiting_reason: &'static str },
	Closed,
}

impl Modal {
	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::Open { waiting_reason } => {
				*self = Modal::Opened { waiting_reason };
				Task::none()
			}
			Message::Close => {
				*self = Modal::Closed;
				Task::none()
			}
			Message::ForceCloseAnyways => window::get_latest().and_then(window::close),
		}
	}

	pub fn view(&self) -> Option<Element<Message>> {
		match self {
			Modal::Closed => None,
			Modal::Opened { waiting_reason } => Some(
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
