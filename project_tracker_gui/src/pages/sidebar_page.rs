use iced::{advanced::widget::Id, alignment::Horizontal, theme, widget::{column, container, row, scrollable, scrollable::RelativeOffset, text_input, Column}, Alignment, Color, Command, Element, Length, Padding, Point, Rectangle};
use iced_drop::find_zones;
use once_cell::sync::Lazy;
use crate::{components::{horizontal_seperator, unfocusable}, core::{Database, DatabaseMessage, TaskId}, project_tracker::UiMessage, styles::SMALL_SPACING_AMOUNT};
use crate::components::{create_new_project_button, loading_screen, overview_button, project_preview, custom_project_preview, EDIT_PROJECT_NAME_TEXT_INPUT_ID, settings_button, toggle_sidebar_button};
use crate::styles::{TextInputStyle, ScrollableStyle, scrollable_vertical_direction, LARGE_TEXT_SIZE, SMALL_PADDING_AMOUNT, PADDING_AMOUNT, SCROLLBAR_WIDTH, SPACING_AMOUNT};
use crate::project_tracker::ProjectTrackerApp;
use crate::core::{OrderedHashMap, ProjectId, Project};

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum SidebarPageMessage {
	OpenCreateNewProject,
	CloseCreateNewProject,
	ChangeCreateNewProjectName(String),

	EditProject(ProjectId),
	StopEditingProject,

	DropTask {
		project_id: ProjectId,
		task_id: TaskId,
		point: Point,
		rect: Rectangle,
	},
	HandleTaskZones {
		project_id: ProjectId,
		task_id: TaskId,
		zones: Vec<(Id, Rectangle)>
	},
	DragTask {
		project_id: ProjectId,
		task_id: TaskId,
		point: Point,
		rect: Rectangle,
	},
	CancelDragTask,
}

impl From<SidebarPageMessage> for UiMessage {
	fn from(value: SidebarPageMessage) -> Self {
		UiMessage::SidebarPageMessage(value)
	}
}

#[derive(Clone)]
pub struct SidebarPage {
	create_new_project_name: Option<String>,
	pub project_being_edited: Option<ProjectId>,
	project_being_task_hovered: Option<ProjectId>,
}

impl SidebarPage {
	pub fn new() -> Self {
		Self {
			create_new_project_name: None,
			project_being_edited: None,
			project_being_task_hovered: None,
		}
	}

	pub fn snap_to_project(&mut self, project_order: usize, database: &Database) -> Command<UiMessage> {
		scrollable::snap_to(
			SCROLLABLE_ID.clone(),
			RelativeOffset {
				x: 0.0,
				y: project_order as f32 / (database.projects.len() as f32 - 1.0),
			}
		)
	}

	fn project_preview_list<'a>(&'a self, projects: &'a OrderedHashMap<ProjectId, Project>, app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		let mut list: Vec<Element<UiMessage>> = projects.iter()
			.map(|(project_id, project)| {
				let selected = match app.selected_project_id {
					Some(selected_project_id) => project_id == selected_project_id,
					None => false,
				};
				let task_hovering = match self.project_being_task_hovered {
					Some(hovered_project_id) => project_id == hovered_project_id,
					None => false
				};
				project_preview(project, project_id, selected, task_hovering)
			})
			.collect();

		if let Some(create_new_project_name) = &self.create_new_project_name {
			let project_name_text_input_element = container(
				unfocusable(
					text_input("New project name", create_new_project_name)
						.id(TEXT_INPUT_ID.clone())
						.size(LARGE_TEXT_SIZE)
						.on_input(|input| SidebarPageMessage::ChangeCreateNewProjectName(input).into())
						.on_submit(DatabaseMessage::CreateProject{
							project_id: ProjectId::generate(),
							name: create_new_project_name.clone()
						}.into())
						.style(theme::TextInput::Custom(Box::new(TextInputStyle))),

					SidebarPageMessage::CloseCreateNewProject.into()
				)
			)
			.width(Length::Fill)
			.align_x(Horizontal::Center)
			.into();

			list.push(custom_project_preview(None, Color::WHITE, 0, 0, project_name_text_input_element, true, false));
		}

		scrollable(
			Column::from_vec(list)
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
				.padding(Padding{ right: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO })
		)
		.id(SCROLLABLE_ID.clone())
		.width(Length::Fill)
		.height(Length::Fill)
		.style(theme::Scrollable::custom(ScrollableStyle))
		.direction(scrollable_vertical_direction())
		.into()
	}

	pub fn update(&mut self, message: SidebarPageMessage, database: &mut Option<Database>) -> Command<UiMessage> {
		match message {
			SidebarPageMessage::OpenCreateNewProject => {
				self.create_new_project_name = Some(String::new());
				Command::batch([
					self.update(SidebarPageMessage::StopEditingProject, database),
					text_input::focus(TEXT_INPUT_ID.clone()),
					scrollable::snap_to(SCROLLABLE_ID.clone(), RelativeOffset::END),
				])
			},
			SidebarPageMessage::CloseCreateNewProject => { self.create_new_project_name = None; Command::none() },
			SidebarPageMessage::ChangeCreateNewProjectName(new_project_name) => { self.create_new_project_name = Some(new_project_name); Command::none() },

			SidebarPageMessage::EditProject(project_id) => {
				self.project_being_edited = Some(project_id);
				Command::batch([
					self.update(SidebarPageMessage::CloseCreateNewProject, database),
					text_input::focus(EDIT_PROJECT_NAME_TEXT_INPUT_ID.clone())
				])
			},
			SidebarPageMessage::StopEditingProject => {
				self.project_being_edited = None;
				Command::none()
			},

			SidebarPageMessage::DropTask { project_id, task_id, .. } => {
				let command = self.project_being_task_hovered.and_then(|dst_project_id| {
					let src_project_id = project_id;
					database.as_mut().map(|db| db.update(DatabaseMessage::MoveTask { task_id, src_project_id, dst_project_id }))
				});
				self.project_being_task_hovered = None;
				command.unwrap_or(Command::none())
			},
			SidebarPageMessage::CancelDragTask => {
				self.project_being_task_hovered = None;
				Command::none()
			},
			SidebarPageMessage::HandleTaskZones{ project_id, zones, .. } => {
				self.project_being_task_hovered = None;
				if let Some(projects) = database.as_ref().map(|db| &db.projects) {
					let source_project_id = project_id;
					for (destination_project_id, _project) in projects.iter() {
						if destination_project_id == source_project_id {
							continue;
						}

						let project_container_id: container::Id = destination_project_id.into();
						let project_widget_id = project_container_id.into();
						for (id, _bounds) in zones.iter() {
							if *id == project_widget_id {
								self.project_being_task_hovered = Some(destination_project_id);
								break;
							}
						}
					}
				}
				Command::none()
			},
			SidebarPageMessage::DragTask { project_id, task_id, rect, .. } => {
				find_zones(
					move |zones| SidebarPageMessage::HandleTaskZones { project_id, task_id, zones }.into(),
					move |zone_bounds| zone_bounds.intersects(&rect),
					None,
					None
				)
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let scrollbar_padding = Padding{ right: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO };

		let list: Element<UiMessage> = if let Some(database) = &app.database {
				self.project_preview_list(&database.projects, app)
			}
			else {
				container(loading_screen())
					.padding(scrollbar_padding)
					.into()
			};

		column![
			container(
				column![
					row![
						overview_button(app.content_page.is_overview_page()),
						toggle_sidebar_button(),
					]
					.align_items(Alignment::Center)
					.spacing(SMALL_SPACING_AMOUNT),

					horizontal_seperator(),
				]
				.spacing(SPACING_AMOUNT)
			)
			.padding(scrollbar_padding),

			list,

			container(
				row![
					settings_button(),

					container(create_new_project_button(self.create_new_project_name.is_none()))
						.width(Length::Fill)
						.align_x(Horizontal::Right),
				]
				.align_items(Alignment::Center)
			)
			.padding(scrollbar_padding),
		]
		.width(Length::Fill)
		.height(Length::Fill)
		.spacing(SPACING_AMOUNT)
		.padding(Padding{ left: PADDING_AMOUNT, right: 0.0, top: PADDING_AMOUNT, bottom: PADDING_AMOUNT })
		.into()
	}
}

impl Default for SidebarPage {
	fn default() -> Self {
		Self::new()
	}
}
