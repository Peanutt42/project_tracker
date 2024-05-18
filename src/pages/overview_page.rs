use iced::{widget::{column, row, container, text}, Element, Length};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

#[derive(Debug, Clone)]
pub struct OverviewPage {

}

impl OverviewPage {
	pub fn new() -> Self {
		Self {

		}
	}

	pub fn view<'a>(&'a self, _app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		column![
			text("Todo Overview").size(35),
			row![
				container(
					text("Todo").size(20)
				)
				.width(Length::FillPortion(1)),
				
				container(
					text("In Progress").size(20)
				)
				.width(Length::FillPortion(1)),
			]
			.width(Length::Fill)
		]
		.spacing(10)
		.into()
	}
}