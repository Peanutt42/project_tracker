use iced::{alignment::Horizontal, widget::{column, container, scrollable, Column}, Element, Length};
use crate::components::{create_new_project_button, loading_screen, overview_button, project_preview, partial_horizontal_seperator, settings_button};
use crate::styles::{HORIZONTAL_PADDING, SPACING_AMOUNT, LARGE_SPACING_AMOUNT};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};
use crate::project::Project;

#[derive(Debug, Clone)]
pub struct SidebarPage {
	
}

impl SidebarPage {
	pub fn new() -> Self {
		Self {
			
		}
	}

	fn project_preview_list<'a>(projects: &'a [Project], app: &'a ProjectTrackerApp, overview_button: Column<'a, UiMessage>) -> Element<'a, UiMessage> {
		let mut list: Vec<Element<UiMessage>> = projects.iter()
		.map(|project| {
			let selected = project.name == app.selected_page_name;
			project_preview(project, selected)
		})
		.collect();
		list.insert(0, overview_button.into());

		scrollable(
			Column::from_vec(list)
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
		)
		.width(Length::Fill)
		.height(Length::Shrink)
		.into()
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let overview_button = column![
			overview_button(app.content_page.is_overview_page()),
			partial_horizontal_seperator(2.5),
		]
		.spacing(LARGE_SPACING_AMOUNT);

		let list: Element<UiMessage> =
			if let Some(saved_state) = &app.saved_state {
				Self::project_preview_list(&saved_state.projects, app, overview_button)
			}
			else {
				column![
					overview_button,
					loading_screen(),
				]
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
				.into()
			};

		column![
			column![
				list,
	
				column![
					partial_horizontal_seperator(2.5),
	
					container(create_new_project_button())
						.align_x(Horizontal::Center)
						.width(Length::Fill),
				]
				.spacing(LARGE_SPACING_AMOUNT)
			]
			.spacing(SPACING_AMOUNT)
			.padding(HORIZONTAL_PADDING),
			
			container(settings_button())
				.height(Length::Fill)
				.align_y(iced::alignment::Vertical::Bottom)
		]
		.into()
	}
}