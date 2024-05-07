use iced::{widget::{button, column, scrollable, text, container}, alignment::{Horizontal, Vertical}, Element, Length};
use iced_aw::{Grid, GridRow, modal};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};
use crate::components::{home_button, project_preview, CreateNewProjectModal, create_new_project_button, CreateNewTaskModal, CreateNewProjectModalMessage, CreateNewTaskModalMessage};



#[derive(Debug, Clone)]
pub enum Page {
	StartPage {
		create_new_project_modal: CreateNewProjectModal,
	},
	ProjectPage {
		project_name: String,
		create_new_task_modal: CreateNewTaskModal,
	},
}

impl Page {
	pub fn update_create_new_project_modal_message(&mut self, message: CreateNewProjectModalMessage) {
		if let Page::StartPage { create_new_project_modal } = self {
			create_new_project_modal.update(message);
		}
	}

	pub fn update_create_new_task_modal_message(&mut self, message: CreateNewTaskModalMessage) {
		if let Page::ProjectPage { create_new_task_modal, .. } = self {
			create_new_task_modal.update(message);
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		match self {
			Page::StartPage { create_new_project_modal } => {
				let project_grid: Element<UiMessage> = if let Some(saved_state) = &app.saved_state {
					let project_rows: Vec<GridRow<UiMessage>> = saved_state.projects
						.chunks(4)
						.map(|project_chunk| {
						let project_row_views: Vec<Element<UiMessage>> = project_chunk
							.iter()
							.map(|project| {
								project_preview(project).into()
							})
							.collect();
						GridRow::with_elements(project_row_views)
					}).collect();

					scrollable(
						Grid::with_rows(project_rows)
							.width(Length::Fill)
							.spacing(10.0)
							.padding(10)
					)
					.width(Length::Fill).into()
				}
				else {
					container(
						text("loading...")
							.horizontal_alignment(Horizontal::Center)
							.vertical_alignment(Vertical::Center)
							.size(50)
					)
					.center_x()
					.center_y()
					.width(Length::Fill)
					.height(Length::Fill)
					.into()
				};

				let background = column![
					create_new_project_button(),
					project_grid,
				];

				modal(background, create_new_project_modal.view())
					.backdrop(UiMessage::CreateNewProjectModalMessage(CreateNewProjectModalMessage::Close))
					.on_esc(UiMessage::CreateNewProjectModalMessage(CreateNewProjectModalMessage::Close))
					.into()
			},
			Page::ProjectPage { project_name, create_new_task_modal } => {
				if let Some(saved_state) = &app.saved_state {
					let mut current_project = None;
					for project in saved_state.projects.iter() {
						if project.name == *project_name {
							current_project = Some(project);
							break;
						}
					}
					let project_element = if let Some(project) = current_project {
						project.view(create_new_task_modal)
					}
					else {
						text("Invalid Project").into()
					};
					column![
						home_button(),
						project_element,
						button("theme").on_press(UiMessage::ToggleTheme)
					].into()
				}
				else {
					column![].into()
				}
			}
		}
	}
}
