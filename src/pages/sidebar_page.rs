use iced::{widget::{scrollable, container, Column, column}, Element, Length, alignment::Horizontal};
use crate::components::{create_new_project_button, loading_screen, overview_button, project_preview, horizontal_seperator, settings_button};
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
				.spacing(10)
		)
		.width(Length::Fill)
		.into()
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let overview_button = column![
			overview_button(app.content_page.is_overview_page()),
			horizontal_seperator(5.0),
		]
		.spacing(10);

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
				.spacing(10)
				.into()
			};

		column![
			container(create_new_project_button())
				.align_x(Horizontal::Right)
				.width(Length::Fill),

			list,

			container(settings_button())
				.height(Length::Fill)
				.align_y(iced::alignment::Vertical::Bottom)
		]
		.spacing(5)
		.into()
	}
}