use iced::{advanced::widget::Id, alignment::Horizontal, theme, widget::{column, container, row, scrollable, scrollable::RelativeOffset, text_input, Column}, Alignment, Color, Command, Element, Length, Padding, Point, Rectangle};
use iced_drop::find_zones;
use once_cell::sync::Lazy;
use crate::{components::{horizontal_seperator, unfocusable}, core::{Database, DatabaseMessage, TaskId, TaskState}, project_tracker::UiMessage, styles::{MINIMAL_DRAG_DISTANCE, SMALL_SPACING_AMOUNT}};
use crate::components::{create_new_project_button, loading_screen, overview_button, project_preview, custom_project_preview, settings_button, toggle_sidebar_button};
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
	CreateNewProject(ProjectId),

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
		task_state: TaskState,
		point: Point,
		rect: Rectangle,
	},
	CancelDragTask,

	DropProject {
		project_id: ProjectId,
		point: Point,
		rect: Rectangle,
	},
	HandleProjectZones {
		project_id: ProjectId,
		zones: Vec<(Id, Rectangle)>,
	},
	DragProject {
		project_id: ProjectId,
		point: Point,
		rect: Rectangle,
	},
	ClickProject(ProjectId),
	CancelDragProject,
}

impl From<SidebarPageMessage> for UiMessage {
	fn from(value: SidebarPageMessage) -> Self {
		UiMessage::SidebarPageMessage(value)
	}
}

#[derive(Clone)]
pub struct SidebarPage {
	create_new_project_name: Option<String>,
	pub task_being_task_hovered: Option<TaskId>,
	project_being_task_hovered: Option<ProjectId>,
	pub project_being_project_hovered: Option<ProjectId>,
	pub dragged_project_id: Option<ProjectId>,
	start_dragging_point: Option<Point>,
	just_minimal_dragging: bool,
	pub pressed_project_id: Option<ProjectId>,
}

impl SidebarPage {
	pub fn new() -> Self {
		Self {
			create_new_project_name: None,
			task_being_task_hovered: None,
			project_being_task_hovered: None,
			project_being_project_hovered: None,
			dragged_project_id: None,
			start_dragging_point: None,
			just_minimal_dragging: true,
			pressed_project_id: None,
		}
	}

	pub fn snap_to_project(&mut self, project_order: usize, database: &Database) -> Command<UiMessage> {
		scrollable::snap_to(
			SCROLLABLE_ID.clone(),
			RelativeOffset {
				x: 0.0,
				y: project_order as f32 / (database.projects().len() as f32 - 1.0),
			}
		)
	}

	pub fn should_select_project(&mut self) -> Option<ProjectId> {
		let project_id_to_select = if self.just_minimal_dragging {
			self.pressed_project_id
		}
		else {
			None
		};

		self.dragged_project_id = None;
		self.start_dragging_point = None;
		self.just_minimal_dragging = false;
		self.pressed_project_id = None;
		self.project_being_project_hovered = None;
		project_id_to_select
	}

	fn project_preview_list<'a>(&'a self, projects: &'a OrderedHashMap<ProjectId, Project>, app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		let mut list: Vec<Element<UiMessage>> = projects.iter()
			.map(|(project_id, project)| {
				let selected = match app.selected_project_id {
					Some(selected_project_id) => project_id == selected_project_id,
					None => false,
				};
				let mut dropzone_highlight = match self.project_being_task_hovered {
					Some(hovered_project_id) => project_id == hovered_project_id,
					None => false
				};
				if let Some(project_being_project_hovered) = &self.project_being_project_hovered {
					if *project_being_project_hovered == project_id {
						dropzone_highlight = true;
					}
				}
				let dragging = match self.dragged_project_id {
					Some(dragged_project_id) => dragged_project_id == project_id,
					None => false,
				};
				project_preview(project, project_id, selected, dropzone_highlight, dragging, self.just_minimal_dragging)
			})
			.collect();

		if let Some(create_new_project_name) = &self.create_new_project_name {
			let project_name_text_input_element = container(
				unfocusable(
					text_input("New project name", create_new_project_name)
						.id(TEXT_INPUT_ID.clone())
						.size(LARGE_TEXT_SIZE)
						.on_input(|input| SidebarPageMessage::ChangeCreateNewProjectName(input).into())
						.on_submit(SidebarPageMessage::CreateNewProject(ProjectId::generate()).into())
						.style(theme::TextInput::Custom(Box::new(TextInputStyle { round_left: true, round_right: true }))),

					SidebarPageMessage::CloseCreateNewProject.into()
				)
			)
			.width(Length::Fill)
			.align_x(Horizontal::Center)
			.into();

			list.push(custom_project_preview(None, None, Color::WHITE, 0, 0, project_name_text_input_element, true, false, false, false));
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
					text_input::focus(TEXT_INPUT_ID.clone()),
					scrollable::snap_to(SCROLLABLE_ID.clone(), RelativeOffset::END),
				])
			},
			SidebarPageMessage::CloseCreateNewProject => { self.create_new_project_name = None; Command::none() },
			SidebarPageMessage::ChangeCreateNewProjectName(new_project_name) => { self.create_new_project_name = Some(new_project_name); Command::none() },
			SidebarPageMessage::CreateNewProject(project_id) => {
				if let Some(db) = database {
					if let Some(create_new_project_name) = &mut self.create_new_project_name {
						return Command::batch([
							db.update(DatabaseMessage::CreateProject {
								project_id,
								name: std::mem::take(create_new_project_name)
							}),
							self.update(SidebarPageMessage::CloseCreateNewProject, database)
						]);
					}
				}
				self.update(SidebarPageMessage::CloseCreateNewProject, database)
			},

			SidebarPageMessage::DropTask { project_id, task_id, .. } => {
				let command = self.project_being_task_hovered
					.and_then(|dst_project_id| {
						let src_project_id = project_id;
						database.as_mut().map(|db| db.update(DatabaseMessage::MoveTask { task_id, src_project_id, dst_project_id }))
					})
					.or(
						self.task_being_task_hovered
							.and_then(|dst_task_id| {
								let src_task_id = task_id;
								database.as_mut().map(|db| db.update(DatabaseMessage::SwapTasks { project_id, task_a_id: src_task_id, task_b_id: dst_task_id }))
							})
					);
				self.project_being_task_hovered = None;
				self.task_being_task_hovered = None;
				command.unwrap_or(Command::none())
			},
			SidebarPageMessage::CancelDragTask => {
				self.project_being_task_hovered = None;
				self.task_being_task_hovered = None;
				Command::none()
			},
			SidebarPageMessage::HandleTaskZones{ zones, .. } => {
				self.project_being_task_hovered = None;
				self.task_being_task_hovered = None;
				let is_hovered = |target_id| {
					for (id, _bounds) in zones.iter() {
						if *id == target_id {
							return true;
						}
					}
					false
				};
				if let Some(projects) = database.as_ref().map(|db| db.projects()) {
					for (dst_project_id, dst_project) in projects.iter() {
						if is_hovered(dst_project.preview_dropzone_id.clone().into()) {
							self.project_being_task_hovered = Some(dst_project_id);
							break;
						}
						for (task_id, task) in dst_project.tasks.iter() {
							if is_hovered(task.dropzone_id.clone().into()) {
								self.task_being_task_hovered = Some(task_id);
								break;
							}
						}
					}
				}
				Command::none()
			},
			SidebarPageMessage::DragTask { project_id, task_id, task_state, point, .. } => {
				let options = Self::project_and_task_dropzone_options(database, project_id, task_id, task_state.is_todo());
				find_zones(
					move |zones| SidebarPageMessage::HandleTaskZones { project_id, task_id, zones }.into(),
					move |zone_bounds| zone_bounds.contains(point),
					options,
					None
				)
			},

			SidebarPageMessage::DropProject { .. } => {
				if let Some(dragged_project_id) = self.dragged_project_id {
					// self.dragged_project_id = None; gets called after LeftClickReleased
					if let Some(hovered_project_id) = self.project_being_project_hovered {
						self.project_being_project_hovered = None;
						if let Some(database) = database {
							return database.update(DatabaseMessage::SwapProjectOrder{
								project_a_id: hovered_project_id,
				 				project_b_id: dragged_project_id,
							});
						}
					}
				}
				self.project_being_project_hovered = None;
				Command::none()
			},
			SidebarPageMessage::DragProject { project_id, point, .. } => {
				self.dragged_project_id = Some(project_id);
				if let Some(start_dragging_point) = self.start_dragging_point {
					if self.just_minimal_dragging {
						self.just_minimal_dragging = start_dragging_point.distance(point) < MINIMAL_DRAG_DISTANCE;
					}
				}
				else {
					self.start_dragging_point = Some(point);
					self.just_minimal_dragging = true;
				}
				let options = Self::project_dropzone_options(database, project_id);
				find_zones(
					move |zones| SidebarPageMessage::HandleProjectZones { project_id, zones }.into(),
				 	move |zone_bounds| zone_bounds.contains(point),
					options,
					None
				)
			},
			SidebarPageMessage::HandleProjectZones { zones, .. } => {
				self.project_being_project_hovered = None;
				if self.dragged_project_id.is_some() {
					if let Some(projects) = database.as_ref().map(|db| db.projects()) {
						for (dst_project_id, dst_project) in projects.iter() {
							let dst_project_widget_id = dst_project.preview_dropzone_id.clone().into();
							for (id, _bounds) in zones.iter() {
								if *id == dst_project_widget_id {
									self.project_being_project_hovered = Some(dst_project_id);
									break;
								}
							}
						}
					}
				}
				Command::none()
			},
			SidebarPageMessage::ClickProject(project_id) => {
				self.pressed_project_id = Some(project_id);
				Command::none()
			},
			SidebarPageMessage::CancelDragProject => {
				self.dragged_project_id = None;
				self.start_dragging_point = None;
				self.just_minimal_dragging = true;
				self.pressed_project_id = None;
				self.project_being_project_hovered = None;
				Command::none()
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let scrollbar_padding = Padding{ right: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO };

		let list: Element<UiMessage> = if let Some(database) = &app.database {
				self.project_preview_list(database.projects(), app)
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

	fn project_dropzone_options(database: &Option<Database>, exception: ProjectId) -> Option<Vec<Id>> {
		database.as_ref().map(|database| {
			database.projects().iter().filter_map(|(project_id, project)| {
				if project_id == exception {
					None
				}
				else {
					Some(project.preview_dropzone_id.clone().into())
				}
			})
			.collect()
		})
	}

	fn project_and_task_dropzone_options(database: &Option<Database>, project_exception: ProjectId, task_exception: TaskId, is_task_todo: bool) -> Option<Vec<Id>> {
		if let Some(database) = database {
			let mut options = Vec::new();
			for (project_id, project) in database.projects().iter() {
				if project_id != project_exception {
					options.push(project.preview_dropzone_id.clone().into());
				}
				if is_task_todo {
					for (task_id, task) in project.tasks.iter() {
						if task_id == task_exception {
							continue;
						}
						options.push(task.dropzone_id.clone().into());
					}
				}
			}
			Some(options)
		}
		else {
			None
		}
	}
}

impl Default for SidebarPage {
	fn default() -> Self {
		Self::new()
	}
}
