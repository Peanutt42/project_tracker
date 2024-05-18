use iced::{widget::{column, container, scrollable, Column}, Element, Length};
use crate::components::{project_preview, create_new_project_button, loading_screen};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

#[derive(Debug, Clone)]
pub struct ProjectListPage {
	
}

impl ProjectListPage {
	pub fn new() -> Self {
		Self {
			
		}
	}
}

impl ProjectListPage {
	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let project_list = if let Some(saved_state) = &app.saved_state {
			let project_previews = saved_state.projects.iter()
				.map(|project| {
					let selected = if let Some(project_page) = &app.project_page {
						project_page.project_name == project.name
					}
					else {
						false
					};
					project_preview(project, selected)
				})
				.collect();
			
			scrollable(
				Column::from_vec(project_previews)
					.width(Length::Fill)
					.spacing(10)
			)
			.width(Length::Fill)
			.into()
		}
		else {
			loading_screen()
		};

		column![
			container(
				create_new_project_button()
			)
			.align_x(iced::alignment::Horizontal::Right)
			.width(Length::Fill),

			project_list,
		]
		.spacing(5)
		.into()
	}
}