use iced::{widget::{scrollable, container, Column, column}, Element, Length, alignment::Horizontal};
use crate::components::{create_new_project_button, loading_screen, overview_button, project_preview, horizontal_seperator};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

#[derive(Debug, Clone)]
pub struct SidebarPage {
	
}

impl SidebarPage {
	pub fn new() -> Self {
		Self {
			
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(saved_state) = &app.saved_state {
			let mut selectables: Vec<Element<UiMessage>> = saved_state.projects.iter()
				.map(|project| {
					let selected = project.name == app.selected_page_name;
					project_preview(project, selected)
				})
				.collect();
				
			selectables.insert(0, column![
				overview_button(app.content_page.is_overview_page()),
				horizontal_seperator(5.0),
			]
			.spacing(10)
			.into());

			column![
				container(create_new_project_button())
					.align_x(Horizontal::Right)
					.width(Length::Fill),

				scrollable(
					Column::from_vec(selectables)
						.width(Length::Fill)
						.spacing(10)
				)
				.width(Length::Fill),
			]
			.spacing(5)
			.into()
		}
		else {
			loading_screen()
		}
	}
}