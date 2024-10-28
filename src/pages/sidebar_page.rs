use crate::components::{
	create_new_project_button, custom_project_preview, loading_screen, project_preview,
	settings_button, stopwatch_button, toggle_sidebar_button,
};
use crate::core::{OrderedHashMap, Project, ProjectId};
use crate::pages::StopwatchPage;
use crate::project_tracker::ProjectTrackerApp;
use crate::styles::{LARGE_TEXT_SIZE, SPACING_AMOUNT};
use crate::{
	components::{
		horizontal_seperator, in_between_dropzone, unfocusable, vertical_scrollable,
		COLOR_PALETTE_BLACK, COLOR_PALETTE_WHITE,
	},
	core::{Database, DatabaseMessage, TaskId},
	pages::StopwatchPageMessage,
	project_tracker::Message,
	styles::{
		text_input_style_default, MINIMAL_DRAG_DISTANCE, PADDING_AMOUNT, SMALL_SPACING_AMOUNT,
	},
};
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
use once_cell::sync::Lazy;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static BOTTOM_PROJECT_DROPZONE_ID: Lazy<container::Id> = Lazy::new(container::Id::unique);
pub static BOTTOM_TODO_TASK_DROPZONE_ID: Lazy<container::Id> = Lazy::new(container::Id::unique);
pub static STOPWATCH_TASK_DROPZONE_ID: Lazy<container::Id> = Lazy::new(container::Id::unique);

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
		filtering_tags: bool,
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

impl From<SidebarPageMessage> for Message {
	fn from(value: SidebarPageMessage) -> Self {
		Message::SidebarPageMessage(value)
	}
}

pub enum SidebarPageAction {
	None,
	Task(Task<Message>),
	SelectProject(ProjectId),
}

impl From<Task<Message>> for SidebarPageAction {
	fn from(value: Task<Message>) -> Self {
		SidebarPageAction::Task(value)
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
pub struct SidebarPage {
	create_new_project_name: Option<String>,
	pub project_dropzone_hovered: Option<ProjectDropzone>,
	pub task_dropzone_hovered: Option<TaskDropzone>,
	pub dragged_project_id: Option<ProjectId>,
	start_dragging_point: Option<Point>,
	just_minimal_dragging: bool,
	pub pressed_project_id: Option<ProjectId>,
}

impl SidebarPage {
	pub const SPLIT_LAYOUT_PERCENTAGE: f32 = 0.3;

	pub fn new() -> Self {
		Self {
			create_new_project_name: None,
			project_dropzone_hovered: None,
			task_dropzone_hovered: None,
			dragged_project_id: None,
			start_dragging_point: None,
			just_minimal_dragging: true,
			pressed_project_id: None,
		}
	}

	pub fn snap_to_project(
		&mut self,
		project_order: usize,
		database: &Database,
	) -> Task<Message> {
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

	pub fn subscription(&self) -> Subscription<SidebarPageMessage> {
		let left_released_subscription =
			iced::event::listen_with(move |event, _status, _id| match event {
				Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
					Some(SidebarPageMessage::LeftClickReleased)
				}
				_ => None,
			});

		let create_new_project_shorcut_subscription =
			keyboard::on_key_press(|key, modifiers| match key.as_ref() {
				keyboard::Key::Character("n") if modifiers.command() && modifiers.shift() => {
					Some(SidebarPageMessage::OpenCreateNewProject)
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
		message: SidebarPageMessage,
		database: &mut Option<Database>,
		stopwatch_page: &mut StopwatchPage,
		is_theme_dark: bool,
	) -> SidebarPageAction {
		match message {
			SidebarPageMessage::OpenCreateNewProject => {
				self.create_new_project_name = Some(String::new());
				Task::batch([
					text_input::focus(TEXT_INPUT_ID.clone()),
					scrollable::snap_to(SCROLLABLE_ID.clone(), RelativeOffset::END),
				])
				.into()
			}
			SidebarPageMessage::CloseCreateNewProject => {
				self.create_new_project_name = None;
				SidebarPageAction::None
			}
			SidebarPageMessage::ChangeCreateNewProjectName(new_project_name) => {
				self.create_new_project_name = Some(new_project_name);
				SidebarPageAction::None
			}
			SidebarPageMessage::CreateNewProject(project_id) => {
				let create_new_project_name = self.create_new_project_name.take();
				if let Some(db) = database {
					if let Some(create_new_project_name) = create_new_project_name {
						db.update(DatabaseMessage::CreateProject {
							project_id,
							name: create_new_project_name,
							color: get_new_project_color(is_theme_dark).into(),
						});
						return SidebarPageAction::SelectProject(project_id);
					}
				}
				SidebarPageAction::None
			}

			SidebarPageMessage::DropTask {
				project_id,
				task_id,
				..
			} => {
				if let Some(hovered_task_dropzone) = self.task_dropzone_hovered {
					match hovered_task_dropzone {
						TaskDropzone::Project(hovered_project_id) => {
							let src_project_id = project_id;
							if let Some(database) = database {
								database.update(DatabaseMessage::MoveTask {
									task_id,
									src_project_id,
									dst_project_id: hovered_project_id,
								});
							}
						}
						TaskDropzone::Task(hovered_task_id) => {
							if let Some(database) = database {
								database.update(DatabaseMessage::MoveTaskBeforeOtherTask {
									project_id,
									task_id,
									other_task_id: hovered_task_id,
								});
							}
						}
						TaskDropzone::EndOfTodoTaskList => {
							if let Some(database) = database {
								database.modify(|projects| {
									if let Some(project) = projects.get_mut(&project_id) {
										project.todo_tasks.move_to_end(&task_id);
									}
								});
							}
						}
						TaskDropzone::Stopwatch => {
							stopwatch_page.update(
								StopwatchPageMessage::Start {
									task: Some((project_id, task_id)),
								},
								database,
								true,
							);
						}
					}
				}
				self.task_dropzone_hovered = None;
				SidebarPageAction::None
			}
			SidebarPageMessage::CancelDragTask => {
				self.task_dropzone_hovered = None;
				SidebarPageAction::None
			}
			SidebarPageMessage::HandleProjectZonesForTasks { zones, .. } => {
				self.task_dropzone_hovered = None;
				if let Some(projects) = database.as_ref().map(|db| db.projects()) {
					for (id, _bounds) in zones.iter() {
						for (dst_project_id, dst_project) in projects.iter() {
							if *id == dst_project.task_dropzone_id.clone().into() {
								self.task_dropzone_hovered =
									Some(TaskDropzone::Project(dst_project_id));
								break;
							}
						}
						if *id == STOPWATCH_TASK_DROPZONE_ID.clone().into() {
							self.task_dropzone_hovered = Some(TaskDropzone::Stopwatch);
						}
					}
				}
				SidebarPageAction::None
			}
			SidebarPageMessage::HandleTaskZones {
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
						for (task_id, task) in project.todo_tasks.iter() {
							if is_hovered(task.dropzone_id.clone().into()) {
								self.task_dropzone_hovered = Some(TaskDropzone::Task(task_id));
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
				SidebarPageAction::None
			}
			SidebarPageMessage::DragTask {
				project_id,
				task_id,
				task_is_todo,
				filtering_tags,
				rect,
				point,
			} => {
				let task_has_needed_time = database
					.as_ref()
					.and_then(|db| {
						db.projects().get(&project_id).and_then(|project| {
							project
								.get_task(&task_id)
								.map(|task| task.needed_time_minutes.is_some())
						})
					})
					.unwrap_or(false);
				let project_options = Self::project_dropzones_for_tasks_options(
					database,
					project_id,
					task_has_needed_time,
				);
				let mut commands = vec![zones_on_point(
					move |zones| {
						SidebarPageMessage::HandleProjectZonesForTasks {
							project_id,
							task_id,
							zones,
						}
						.into()
					},
					point,
					project_options,
					None,
				)];
				if task_is_todo && !filtering_tags {
					let task_options = Self::task_dropzone_options(database, project_id, task_id);
					commands.push(find_zones(
						move |zones| {
							SidebarPageMessage::HandleTaskZones {
								project_id,
								task_id,
								zones,
							}
							.into()
						},
						move |zone_bounds| zone_bounds.intersects(&rect),
						task_options,
						None,
					));
				}
				Task::batch(commands).into()
			}

			SidebarPageMessage::DropProject { .. } => {
				if let Some(dragged_project_id) = self.dragged_project_id {
					// self.dragged_project_id = None; gets called after LeftClickReleased
					if let Some(project_dropzone_hovered) = self.project_dropzone_hovered {
						self.project_dropzone_hovered = None;
						if let Some(database) = database {
							match project_dropzone_hovered {
								ProjectDropzone::Project(hovered_project_id) => {
									database.update(
										DatabaseMessage::MoveProjectBeforeOtherProject {
											project_id: dragged_project_id,
											other_project_id: hovered_project_id,
										},
									);
								}
								ProjectDropzone::EndOfList => {
									database.update(DatabaseMessage::MoveProjectToEnd(
										dragged_project_id,
									));
								}
							}
						}
					}
				}
				self.project_dropzone_hovered = None;
				SidebarPageAction::None
			}
			SidebarPageMessage::DragProject {
				project_id,
				point,
				rect,
			} => {
				self.dragged_project_id = Some(project_id);
				if let Some(start_dragging_point) = self.start_dragging_point {
					if self.just_minimal_dragging {
						self.just_minimal_dragging =
							start_dragging_point.distance(point) < MINIMAL_DRAG_DISTANCE;
					}
				} else {
					self.start_dragging_point = Some(point);
					self.just_minimal_dragging = true;
				}
				let options = Self::project_dropzone_options(database, project_id);
				find_zones(
					move |zones| {
						SidebarPageMessage::HandleProjectZones { project_id, zones }.into()
					},
					move |zone_bounds| zone_bounds.intersects(&rect),
					options,
					None,
				)
				.into()
			}
			SidebarPageMessage::HandleProjectZones { zones, .. } => {
				self.project_dropzone_hovered = None;
				if self.dragged_project_id.is_some() {
					if let Some(projects) = database.as_ref().map(|db| db.projects()) {
						let bottom_project_dropzone_widget_id =
							BOTTOM_PROJECT_DROPZONE_ID.clone().into();

						for (dst_project_id, dst_project) in projects.iter() {
							let dst_project_widget_id =
								dst_project.project_dropzone_id.clone().into();
							for (id, _bounds) in zones.iter() {
								if *id == dst_project_widget_id {
									self.project_dropzone_hovered =
										Some(ProjectDropzone::Project(dst_project_id));
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
				SidebarPageAction::None
			}
			SidebarPageMessage::ClickProject(project_id) => {
				self.pressed_project_id = Some(project_id);
				SidebarPageAction::None
			}
			SidebarPageMessage::CancelDragProject => {
				self.dragged_project_id = None;
				self.start_dragging_point = None;
				self.just_minimal_dragging = true;
				self.pressed_project_id = None;
				self.project_dropzone_hovered = None;
				SidebarPageAction::None
			}

			SidebarPageMessage::LeftClickReleased => self
				.should_select_project()
				.map(SidebarPageAction::SelectProject)
				.unwrap_or(SidebarPageAction::None),
		}
	}

	fn project_preview_list<'a>(
		&'a self,
		projects: &'a OrderedHashMap<ProjectId, Project>,
		app: &'a ProjectTrackerApp,
	) -> Element<'a, Message> {
		let mut list: Vec<Element<Message>> = projects
			.iter()
			.map(|(project_id, project)| {
				let selected = match &app.project_page {
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
				project_preview(
					project,
					project_id,
					selected,
					project_dropzone_highlight,
					task_dropzone_highlight,
					dragging,
					self.just_minimal_dragging,
				)
			})
			.collect();

		let end_of_list_dropzone_hovered = match self.project_dropzone_hovered {
			Some(dropzone_hovered) => matches!(dropzone_hovered, ProjectDropzone::EndOfList),
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
					.on_input(|input| SidebarPageMessage::ChangeCreateNewProjectName(input).into())
					.on_submit(SidebarPageMessage::CreateNewProject(ProjectId::generate()).into())
					.style(text_input_style_default),
				SidebarPageMessage::CloseCreateNewProject.into(),
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

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<Message> {
		let list: Element<Message> = if let Some(database) = &app.database {
			self.project_preview_list(database.projects(), app)
		} else {
			loading_screen()
		};

		column![
			column![
				row![
					stopwatch_button(
						&app.stopwatch_page,
						app.project_page.is_none(),
						matches!(self.task_dropzone_hovered, Some(TaskDropzone::Stopwatch))
					),
					toggle_sidebar_button(true),
				]
				.align_y(Alignment::Center)
				.spacing(SMALL_SPACING_AMOUNT),
				horizontal_seperator(),
			]
			.spacing(SPACING_AMOUNT)
			.padding(Padding {
				left: PADDING_AMOUNT,
				right: PADDING_AMOUNT,
				top: PADDING_AMOUNT,
				bottom: 0.0, // project list already has a dropzone padding/spacing
			}),
			list,
			row![
				settings_button(),
				container(create_new_project_button(
					self.create_new_project_name.is_none()
				))
				.width(Fill)
				.align_x(Horizontal::Right),
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
		database: &Option<Database>,
		exception: ProjectId,
	) -> Option<Vec<Id>> {
		// the dropzone of the project below the exception project does not make sense as a option,
		// since the exception project is already before the project below it
		let mut skip_project_order = None;

		database.as_ref().map(|database| {
			let mut options: Vec<Id> = database
				.projects()
				.iter()
				.enumerate()
				.filter_map(|(i, (project_id, project))| {
					if project_id == exception {
						skip_project_order = Some(i + 1);
						None
					} else {
						match skip_project_order {
							Some(skip_order) => {
								if i == skip_order {
									skip_project_order = None;
									None
								} else {
									Some(project.project_dropzone_id.clone().into())
								}
							}
							None => Some(project.project_dropzone_id.clone().into()),
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
		database: &Option<Database>,
		exception: ProjectId,
		task_has_needed_time: bool,
	) -> Option<Vec<Id>> {
		database.as_ref().map(|database| {
			let mut options: Vec<Id> = database
				.projects()
				.iter()
				.filter_map(|(project_id, project)| {
					if project_id == exception {
						None
					} else {
						Some(project.task_dropzone_id.clone().into())
					}
				})
				.collect();

			if task_has_needed_time {
				options.push(STOPWATCH_TASK_DROPZONE_ID.clone().into());
			}

			options
		})
	}

	fn task_dropzone_options(
		database: &Option<Database>,
		project_exception: ProjectId,
		task_exception: TaskId,
	) -> Option<Vec<Id>> {
		if let Some(database) = database {
			let mut options = Vec::new();

			for (project_id, project) in database.projects().iter() {
				if project_id == project_exception {
					let last_task_id = project
						.todo_tasks
						.get_key_at_order(project.todo_tasks.len() - 1);
					let mut skip_task_order = None;

					for (i, (task_id, task)) in project.todo_tasks.iter().enumerate() {
						if task_id == task_exception {
							skip_task_order = Some(i + 1);
						} else {
							match skip_task_order {
								Some(skip_order) if i == skip_order => skip_task_order = None,
								_ => options.push(task.dropzone_id.clone().into()),
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

			Some(options)
		} else {
			None
		}
	}
}

impl Default for SidebarPage {
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
