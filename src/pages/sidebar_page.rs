use crate::components::{
	create_new_project_button, custom_project_preview, loading_screen, overview_button,
	project_preview, settings_button, stopwatch_button, toggle_sidebar_button,
	LARGE_LOADING_SPINNER_SIZE,
};
use crate::core::{IcedColorConversion, ProjectUiIdMap, TaskUiIdMap};
use crate::styles::{LARGE_TEXT_SIZE, SPACING_AMOUNT};
use crate::synchronization::SynchronizationError;
use crate::DatabaseState;
use crate::{
	components::{
		horizontal_seperator, in_between_dropzone, unfocusable, vertical_scrollable,
		COLOR_PALETTE_BLACK, COLOR_PALETTE_WHITE,
	},
	pages::stopwatch_page,
	project_tracker::{self, ProjectTrackerApp},
	styles::{
		text_input_style_default, MINIMAL_DRAG_DISTANCE, PADDING_AMOUNT, SMALL_SPACING_AMOUNT,
	},
};
use iced::widget::Space;
use iced::{
	advanced::widget::Id,
	alignment::Horizontal,
	keyboard, mouse,
	widget::{
		column, container, row,
		scrollable::{self, RelativeOffset},
		text_input, Column,
	},
	Alignment, Color, Element, Event,
	Length::Fill,
	Padding, Point, Rectangle, Subscription, Task,
};
use iced_drop::{find_zones, zones_on_point};
use project_tracker_core::{
	Database, DatabaseMessage, OrderedHashMap, Project, ProjectId, SerializableColor, SortMode,
	TaskId,
};
use std::sync::{Arc, LazyLock};

static SCROLLABLE_ID: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);
static TEXT_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);
static BOTTOM_PROJECT_DROPZONE_ID: LazyLock<container::Id> = LazyLock::new(container::Id::unique);
pub static BOTTOM_TODO_TASK_DROPZONE_ID: LazyLock<container::Id> =
	LazyLock::new(container::Id::unique);
pub static STOPWATCH_TASK_DROPZONE_ID: LazyLock<container::Id> =
	LazyLock::new(container::Id::unique);

#[derive(Clone, Debug)]
pub enum Message {
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
	/// Handles Project Dropzones only for tasks being dropped onto them
	HandleProjectZonesForTasks {
		project_id: ProjectId,
		task_id: TaskId,
		zones: Vec<(Id, Rectangle)>,
	},
	HandleTaskZones {
		project_id: ProjectId,
		task_id: TaskId,
		zones: Vec<(Id, Rectangle)>,
	},
	DragTask {
		project_id: ProjectId,
		task_id: TaskId,
		task_is_todo: bool,
		filtering_tasks: bool,
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

	LeftClickReleased,
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		project_tracker::Message::SidebarPageMessage(value)
	}
}

pub enum Action {
	None,
	Actions(Vec<Action>),
	Task(Task<Message>),
	DatabaseMessage(DatabaseMessage),
	StopwatchPageMessage(stopwatch_page::Message),
	SelectProject(ProjectId),
}

impl From<Task<Message>> for Action {
	fn from(value: Task<Message>) -> Self {
		Action::Task(value)
	}
}
impl From<DatabaseMessage> for Action {
	fn from(value: DatabaseMessage) -> Self {
		Action::DatabaseMessage(value)
	}
}
impl From<stopwatch_page::Message> for Action {
	fn from(value: stopwatch_page::Message) -> Self {
		Action::StopwatchPageMessage(value)
	}
}

fn get_new_project_color(is_theme_dark: bool) -> Color {
	if is_theme_dark {
		COLOR_PALETTE_WHITE
	} else {
		COLOR_PALETTE_BLACK
	}
}

#[derive(Clone)]
pub struct Page {
	create_new_project_name: Option<String>,
	pub project_dropzone_hovered: Option<ProjectDropzone>,
	pub task_dropzone_hovered: Option<TaskDropzone>,
	pub dragged_project_id: Option<ProjectId>,
	start_dragging_point: Option<Point>,
	just_minimal_dragging: bool,
	pub pressed_project_id: Option<ProjectId>,
	pub synchronization_error: Option<Arc<SynchronizationError>>,
}

impl Page {
	pub const DEFAULT_SPLIT_RATIO: f32 = 0.3;

	pub fn new() -> Self {
		Self {
			create_new_project_name: None,
			project_dropzone_hovered: None,
			task_dropzone_hovered: None,
			dragged_project_id: None,
			start_dragging_point: None,
			just_minimal_dragging: true,
			pressed_project_id: None,
			synchronization_error: None,
		}
	}

	pub fn snap_to_project(&mut self, project_order: usize, database: &Database) -> Task<Message> {
		scrollable::snap_to(
			SCROLLABLE_ID.clone(),
			RelativeOffset {
				x: 0.0,
				y: project_order as f32 / (database.projects().len() as f32 - 1.0),
			},
		)
	}

	pub fn should_select_project(&mut self) -> Option<ProjectId> {
		let project_id_to_select = if self.just_minimal_dragging {
			self.pressed_project_id
		} else {
			None
		};

		self.dragged_project_id = None;
		self.start_dragging_point = None;
		self.just_minimal_dragging = true;
		self.pressed_project_id = None;
		self.project_dropzone_hovered = None;
		project_id_to_select
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let left_released_subscription =
			iced::event::listen_with(move |event, _status, _id| match event {
				Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
					Some(Message::LeftClickReleased)
				}
				_ => None,
			});

		let create_new_project_shorcut_subscription =
			keyboard::on_key_press(|key, modifiers| match key.as_ref() {
				keyboard::Key::Character("n") if modifiers.command() && modifiers.shift() => {
					Some(Message::OpenCreateNewProject)
				}
				_ => None,
			});

		Subscription::batch([
			left_released_subscription,
			create_new_project_shorcut_subscription,
		])
	}

	#[must_use]
	pub fn update(
		&mut self,
		message: Message,
		database: Option<&Database>,
		project_ui_ids: &mut ProjectUiIdMap,
		task_ui_ids: &mut TaskUiIdMap,
		is_theme_dark: bool,
	) -> Action {
		match message {
			Message::OpenCreateNewProject => {
				self.create_new_project_name = Some(String::new());
				Task::batch([
					text_input::focus(TEXT_INPUT_ID.clone()),
					scrollable::snap_to(SCROLLABLE_ID.clone(), RelativeOffset::END),
				])
				.into()
			}
			Message::CloseCreateNewProject => {
				self.create_new_project_name = None;
				Action::None
			}
			Message::ChangeCreateNewProjectName(new_project_name) => {
				self.create_new_project_name = Some(new_project_name);
				Action::None
			}
			Message::CreateNewProject(project_id) => match self.create_new_project_name.take() {
				Some(create_new_project_name) => Action::Actions(vec![
					DatabaseMessage::CreateProject {
						project_id,
						name: create_new_project_name,
						color: SerializableColor::from_iced_color(get_new_project_color(
							is_theme_dark,
						)),
					}
					.into(),
					Action::SelectProject(project_id),
				]),
				None => Action::None,
			},

			Message::DropTask {
				project_id,
				task_id,
				..
			} => match self.task_dropzone_hovered {
				Some(hovered_task_dropzone) => {
					self.task_dropzone_hovered = None;
					match hovered_task_dropzone {
						TaskDropzone::Project(hovered_project_id) => DatabaseMessage::MoveTask {
							task_id,
							src_project_id: project_id,
							dst_project_id: hovered_project_id,
						}
						.into(),

						TaskDropzone::Task(hovered_task_id) => {
							DatabaseMessage::MoveTaskBeforeOtherTask {
								project_id,
								task_id,
								other_task_id: hovered_task_id,
							}
							.into()
						}

						TaskDropzone::EndOfTodoTaskList => DatabaseMessage::MoveTodoTaskToEnd {
							project_id,
							task_id,
						}
						.into(),

						TaskDropzone::Stopwatch => stopwatch_page::Message::StopTask {
							project_id,
							task_id,
						}
						.into(),
					}
				}
				None => Action::None,
			},
			Message::CancelDragTask => {
				self.task_dropzone_hovered = None;
				Action::None
			}
			Message::HandleProjectZonesForTasks { zones, .. } => {
				self.task_dropzone_hovered = None;
				if let Some(projects) = database.as_ref().map(|db| db.projects()) {
					for (id, _bounds) in zones.iter() {
						for dst_project_id in projects.keys() {
							if *id
								== project_ui_ids
									.get_task_dropzone_id_mut(*dst_project_id)
									.into()
							{
								self.task_dropzone_hovered =
									Some(TaskDropzone::Project(*dst_project_id));
								break;
							}
						}
						if *id == STOPWATCH_TASK_DROPZONE_ID.clone().into() {
							self.task_dropzone_hovered = Some(TaskDropzone::Stopwatch);
						}
					}
				}
				Action::None
			}
			Message::HandleTaskZones {
				zones, project_id, ..
			} => {
				if !zones.is_empty()
					&& !matches!(self.task_dropzone_hovered, Some(TaskDropzone::Project(_)))
				{
					self.task_dropzone_hovered = None;
					let is_hovered = |target_id| {
						for (id, _bounds) in zones.iter() {
							if *id == target_id {
								return true;
							}
						}
						false
					};
					if let Some(project) =
						database.as_ref().and_then(|db| db.get_project(&project_id))
					{
						for task_id in project.todo_tasks.keys() {
							if is_hovered(task_ui_ids.get_dropzone_id_mut(*task_id).into()) {
								self.task_dropzone_hovered = Some(TaskDropzone::Task(*task_id));
								break;
							}
						}
						if is_hovered(BOTTOM_TODO_TASK_DROPZONE_ID.clone().into()) {
							self.task_dropzone_hovered = Some(TaskDropzone::EndOfTodoTaskList);
						}
						if is_hovered(STOPWATCH_TASK_DROPZONE_ID.clone().into()) {
							self.task_dropzone_hovered = Some(TaskDropzone::Stopwatch);
						}
					}
				}
				Action::None
			}
			Message::DragTask {
				project_id,
				task_id,
				task_is_todo,
				filtering_tasks,
				rect,
				point,
			} => {
				let project_options =
					Self::project_dropzones_for_tasks_options(database, project_id, project_ui_ids);
				let mut commands = vec![zones_on_point(
					move |zones| Message::HandleProjectZonesForTasks {
						project_id,
						task_id,
						zones,
					},
					point,
					project_options,
					None,
				)];
				if task_is_todo && !filtering_tasks {
					let task_options =
						Self::task_dropzone_options(database, project_id, task_id, task_ui_ids);
					commands.push(find_zones(
						move |zones| Message::HandleTaskZones {
							project_id,
							task_id,
							zones,
						},
						move |zone_bounds| zone_bounds.intersects(&rect),
						task_options,
						None,
					));
				}
				Task::batch(commands).into()
			}

			Message::DropProject { .. } => {
				if let Some(dragged_project_id) = self.dragged_project_id {
					// self.dragged_project_id = None; gets called after LeftClickReleased
					if let Some(project_dropzone_hovered) = self.project_dropzone_hovered {
						self.project_dropzone_hovered = None;
						return match project_dropzone_hovered {
							ProjectDropzone::Project(hovered_project_id) => {
								DatabaseMessage::MoveProjectBeforeOtherProject {
									project_id: dragged_project_id,
									other_project_id: hovered_project_id,
								}
								.into()
							}

							ProjectDropzone::EndOfList => {
								DatabaseMessage::MoveProjectToEnd(dragged_project_id).into()
							}
						};
					}
				}
				Action::None
			}
			Message::DragProject {
				project_id,
				point,
				rect,
			} => {
				self.dragged_project_id = Some(project_id);
				match self.start_dragging_point {
					Some(start_dragging_point) => {
						if self.just_minimal_dragging {
							self.just_minimal_dragging =
								start_dragging_point.distance(point) < MINIMAL_DRAG_DISTANCE;
						}
					}
					None => {
						self.start_dragging_point = Some(point);
						self.just_minimal_dragging = true;
					}
				}
				let options = Self::project_dropzone_options(database, project_id, project_ui_ids);
				find_zones(
					move |zones| Message::HandleProjectZones { project_id, zones },
					move |zone_bounds| zone_bounds.intersects(&rect),
					options,
					None,
				)
				.into()
			}
			Message::HandleProjectZones { zones, .. } => {
				self.project_dropzone_hovered = None;
				if self.dragged_project_id.is_some() {
					if let Some(projects) = database.as_ref().map(|db| db.projects()) {
						let bottom_project_dropzone_widget_id =
							BOTTOM_PROJECT_DROPZONE_ID.clone().into();

						for dst_project_id in projects.keys() {
							let dst_project_widget_id = project_ui_ids
								.get_project_dropzone_id_mut(*dst_project_id)
								.into();
							for (id, _bounds) in zones.iter() {
								if *id == dst_project_widget_id {
									self.project_dropzone_hovered =
										Some(ProjectDropzone::Project(*dst_project_id));
									break;
								}
								if *id == bottom_project_dropzone_widget_id {
									self.project_dropzone_hovered =
										Some(ProjectDropzone::EndOfList);
									break;
								}
							}
						}
					}
				}
				Action::None
			}
			Message::ClickProject(project_id) => {
				self.pressed_project_id = Some(project_id);
				Action::None
			}
			Message::CancelDragProject => {
				self.dragged_project_id = None;
				self.start_dragging_point = None;
				self.just_minimal_dragging = true;
				self.pressed_project_id = None;
				self.project_dropzone_hovered = None;
				Action::None
			}

			Message::LeftClickReleased => self
				.should_select_project()
				.map(Action::SelectProject)
				.unwrap_or(Action::None),
		}
	}

	fn project_preview_list<'a>(
		&'a self,
		projects: &'a OrderedHashMap<ProjectId, Project>,
		app: &'a ProjectTrackerApp,
	) -> Element<'a, project_tracker::Message> {
		let mut list: Vec<Element<project_tracker::Message>> = projects
			.iter()
			.map(|(project_id, project)| {
				let selected = match &app.content_page.project_page {
					Some(project_page) => project_id == project_page.project_id,
					None => false,
				};
				let project_dropzone_highlight = match self.project_dropzone_hovered {
					Some(ProjectDropzone::Project(hovered_project_id)) => {
						hovered_project_id == project_id
					}
					_ => false,
				};
				let task_dropzone_highlight = match self.task_dropzone_hovered {
					Some(TaskDropzone::Project(hovered_project_id)) => {
						project_id == hovered_project_id
					}
					_ => false,
				};
				let dragging = match self.dragged_project_id {
					Some(dragged_project_id) => dragged_project_id == project_id,
					None => false,
				};
				let (project_dropzone_id, task_dropzone_id) = app
					.project_ui_id_map
					.get_project_task_dropzone_ids(project_id);
				project_preview(
					project,
					project_id,
					project_dropzone_id,
					task_dropzone_id,
					selected,
					project_dropzone_highlight,
					task_dropzone_highlight,
					dragging,
					self.just_minimal_dragging,
				)
			})
			.collect();

		let end_of_list_dropzone_hovered = match self.project_dropzone_hovered {
			Some(dropzone_hovered) => {
				matches!(dropzone_hovered, ProjectDropzone::EndOfList)
			}
			None => false,
		};

		list.push(in_between_dropzone(
			BOTTOM_PROJECT_DROPZONE_ID.clone(),
			end_of_list_dropzone_hovered,
		));

		if let Some(create_new_project_name) = &self.create_new_project_name {
			let project_name_text_input_element = container(unfocusable(
				text_input("New project name", create_new_project_name)
					.id(TEXT_INPUT_ID.clone())
					.size(LARGE_TEXT_SIZE)
					.on_input(|input| Message::ChangeCreateNewProjectName(input).into())
					.on_submit(Message::CreateNewProject(ProjectId::generate()).into())
					.style(text_input_style_default),
				Message::CloseCreateNewProject.into(),
			))
			.width(Fill)
			.align_x(Horizontal::Center)
			.into();

			list.push(custom_project_preview(
				None,
				None,
				None,
				get_new_project_color(app.is_theme_dark()),
				0,
				0,
				project_name_text_input_element,
				true,
				false,
				false,
				false,
				false,
			));
		}

		vertical_scrollable(Column::from_vec(list).width(Fill))
			.id(SCROLLABLE_ID.clone())
			.height(Fill)
			.into()
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, project_tracker::Message> {
		let list: Element<project_tracker::Message> = match &app.database {
			DatabaseState::Loaded(database) => self.project_preview_list(database.projects(), app),
			_ => container(loading_screen(LARGE_LOADING_SPINNER_SIZE))
				.center(Fill)
				.into(),
		};

		let synchronization_error_view = match &self.synchronization_error {
			Some(error) if app.synchronization.is_some() => error.view(),
			_ => Space::new(Fill, 0.0).into(),
		};

		column![
			column![
				row![
					overview_button(app.content_page.is_overview_page_opened()),
					toggle_sidebar_button(true),
				]
				.align_y(Alignment::Center)
				.spacing(SMALL_SPACING_AMOUNT),
				horizontal_seperator(),
			]
			.spacing(SPACING_AMOUNT)
			.padding(PADDING_AMOUNT),
			column![
				stopwatch_button(
					&app.content_page.stopwatch_page,
					app.content_page.is_stopwatch_page_opened(),
					matches!(self.task_dropzone_hovered, Some(TaskDropzone::Stopwatch)),
					app.database.ok(),
				),
				horizontal_seperator(),
			]
			.spacing(SPACING_AMOUNT)
			.padding(
				Padding::default()
					.left(PADDING_AMOUNT)
					.right(PADDING_AMOUNT)
			),
			list,
			row![
				settings_button(),
				synchronization_error_view,
				create_new_project_button(self.create_new_project_name.is_none())
			]
			.align_y(Alignment::Center)
			.padding(Padding::new(PADDING_AMOUNT)),
		]
		.width(Fill)
		.height(Fill)
		// .spacing(SPACING_AMOUNT) this is not needed since every project in the list has a SPACING_AMOUNT height dropzone
		.into()
	}

	fn project_dropzone_options(
		database: Option<&Database>,
		exception: ProjectId,
		project_ui_ids: &mut ProjectUiIdMap,
	) -> Option<Vec<Id>> {
		// the dropzone of the project below the exception project does not make sense as a option,
		// since the exception project is already before the project below it
		let mut skip_project_order = None;

		database.as_ref().map(|database| {
			let mut options: Vec<Id> = database
				.projects()
				.keys()
				.enumerate()
				.filter_map(|(i, project_id)| {
					if *project_id == exception {
						skip_project_order = Some(i + 1);
						None
					} else {
						match skip_project_order {
							Some(skip_order) => {
								if i == skip_order {
									skip_project_order = None;
									None
								} else {
									Some(
										project_ui_ids
											.get_project_dropzone_id_mut(*project_id)
											.into(),
									)
								}
							}
							None => Some(
								project_ui_ids
									.get_project_dropzone_id_mut(*project_id)
									.into(),
							),
						}
					}
				})
				.collect();

			if let Some(last_project_id) = database
				.projects()
				.get_key_at_order(database.projects().len() - 1)
			{
				if *last_project_id != exception {
					options.push(BOTTOM_PROJECT_DROPZONE_ID.clone().into());
				}
			}

			options
		})
	}

	fn project_dropzones_for_tasks_options(
		database: Option<&Database>,
		exception: ProjectId,
		project_ui_ids: &mut ProjectUiIdMap,
	) -> Option<Vec<Id>> {
		database.as_ref().map(|database| {
			let mut options: Vec<Id> = database
				.projects()
				.keys()
				.filter_map(|project_id| {
					if *project_id == exception {
						None
					} else {
						Some(project_ui_ids.get_task_dropzone_id_mut(*project_id).into())
					}
				})
				.collect();

			options.push(STOPWATCH_TASK_DROPZONE_ID.clone().into());

			options
		})
	}

	fn task_dropzone_options(
		database: Option<&Database>,
		project_id: ProjectId,
		task_exception: TaskId,
		task_ui_ids: &mut TaskUiIdMap,
	) -> Option<Vec<Id>> {
		database.map(|database| {
			let mut options = Vec::new();

			if let Some(project) = database.get_project(&project_id) {
				if matches!(project.sort_mode, SortMode::Manual) {
					let last_task_id = project
						.todo_tasks
						.get_key_at_order(project.todo_tasks.len() - 1);
					let mut skip_task_order = None;

					for (i, task_id) in project.todo_tasks.keys().enumerate() {
						if *task_id == task_exception {
							skip_task_order = Some(i + 1);
						} else {
							match skip_task_order {
								Some(skip_order) if i == skip_order => skip_task_order = None,
								_ => options.push(task_ui_ids.get_dropzone_id_mut(*task_id).into()),
							}
						}
					}
					if let Some(last_task_id) = last_task_id {
						if task_exception != *last_task_id {
							options.push(BOTTOM_TODO_TASK_DROPZONE_ID.clone().into());
						}
					}
				}
			}
			options
		})
	}
}

impl Default for Page {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ProjectDropzone {
	Project(ProjectId),
	EndOfList,
}

#[derive(Debug, Clone, Copy)]
pub enum TaskDropzone {
	Project(ProjectId),
	Task(TaskId),
	EndOfTodoTaskList,
	Stopwatch,
}
