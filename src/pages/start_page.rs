use iced::{widget::{column, scrollable, container}, Element, Length};
use iced_aw::{modal, Grid, GridRow, Spinner};
use crate::components::{project_preview, CreateNewProjectModal, create_new_project_button, CreateNewProjectModalMessage};
use crate::pages::Page;
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

#[derive(Debug, Clone)]
pub struct StartPage {
	create_new_project_modal: CreateNewProjectModal,
}

impl StartPage {
	pub fn new() -> Self {
		Self {
			create_new_project_modal: CreateNewProjectModal::new(),
		}
	}
}

impl Page for StartPage {
	fn update_create_new_project_modal(&mut self, message: CreateNewProjectModalMessage) {
		self.create_new_project_modal.update(message);
	}

	fn update_create_new_task_modal(&mut self, _message: crate::components::CreateNewTaskModalMessage) {}

	fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let dark_mode = app.is_dark_mode();

		let project_grid: Element<UiMessage> = if let Some(saved_state) = &app.saved_state {
			let project_rows: Vec<GridRow<UiMessage>> = saved_state.projects
				.chunks(4)
				.map(|project_chunk| {
				let project_row_views: Vec<Element<UiMessage>> = project_chunk
					.iter()
					.map(|project| project_preview(project))
					.collect();
				GridRow::with_elements(project_row_views)
			}).collect();

			scrollable(
				Grid::with_rows(project_rows)
					.width(Length::Fill)
					.spacing(15.0)
					.padding(10)
			)
			.width(Length::Fill)
			.into()
		}
		else {
			container(
				Spinner::new()
					.width(Length::Fixed(75.0))
					.height(Length::Fixed(75.0)).circle_radius(3.0)
			)
			.width(Length::Fill)
			.height(Length::Fill)
			.center_x()
			.center_y()
			.into()
		};

		let background = column![
			create_new_project_button(),
			project_grid,
		];

		modal(background, self.create_new_project_modal.view(dark_mode))
			.backdrop(CreateNewProjectModalMessage::Close.into())
			.on_esc(CreateNewProjectModalMessage::Close.into())
			.into()
	}
}