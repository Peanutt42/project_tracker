use crate::{
	components::{
		cancel_search_tasks_button, color_palette, completion_bar, create_new_task_button, edit_color_palette_button, edit_project_name_button, horizontal_scrollable, project_context_menu_button, search_tasks_button, task_list, task_tag_button, unfocusable, ScalarAnimation, CREATE_NEW_TASK_NAME_INPUT_ID, EDIT_DUE_DATE_TEXT_INPUT_ID, EDIT_NEEDED_TIME_TEXT_INPUT_ID, HORIZONTAL_SCROLLABLE_PADDING, TASK_LIST_ID
	},
	core::{
		generate_task_id, Database, DatabaseMessage, Preferences, Project, ProjectId,
		SerializableDate, Task, TaskId, TaskTagId,
	},
	icons::{icon_to_char, Bootstrap, BOOTSTRAP_FONT},
	project_tracker::{ProjectTrackerApp, Message},
	styles::{
		text_input_style_default, text_input_style_only_round_left, LARGE_PADDING_AMOUNT,
		MINIMAL_DRAG_DISTANCE, PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT,
		TITLE_TEXT_SIZE,
	},
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use iced::{
	alignment::{Alignment, Horizontal},
	keyboard, mouse,
	widget::{
		column, container, row,
		scrollable::{self, RelativeOffset},
		text, text_editor, text_input, Row, Space,
	},
	Color, Element, Event,
	Length::Fill,
	Padding, Point, Subscription,
};
use iced_aw::{drop_down, DropDown};
use once_cell::sync::Lazy;
use std::{
	collections::HashSet,
	fs::File,
	io::{self, BufRead},
	time::Instant,
};
use walkdir::WalkDir;

static PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static SEARCH_TASKS_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum ProjectPageMessage {
	OpenCreateNewTask,
	CloseCreateNewTask,
	ChangeCreateNewTaskName(String),
	ToggleCreateNewTaskTag(TaskTagId),
	CreateNewTask,

	ShowContextMenu,
	HideContextMenu,
	OpenManageTaskTagsModal,

	OpenSearchTasks,
	CloseSearchTasks,
	ChangeSearchTasksFilter(String),

	ImportSourceCodeTodosDialog,
	ImportSourceCodeTodos(Vec<Task>),
	ImportSourceCodeTodosDialogCanceled,

	ShowSourceCodeTodos(bool),

	ShowDoneTasks(bool),

	ToggleFilterTaskTag(TaskTagId),
	UnsetFilterTaskTag(TaskTagId),

	ShowColorPicker,
	HideColorPicker,
	ChangeProjectColor(Color),

	EditProjectName,
	StopEditingProjectName,
	ChangeEditedProjectName(String),
	ChangeProjectName,

	ConfirmDeleteProject,

	EditTask(TaskId),
	StopEditingTask,
	TaskNameAction(text_editor::Action),
	ToggleTaskTag(TaskTagId),
	EditTaskNeededTime,
	ClearTaskNeededTime,
	ChangeNewTaskNeededTimeInput(Option<usize>),
	InvalidNeededTimeInput,
	ChangeTaskNeededTime,
	EditTaskDueDate,
	ChangeTaskDueDate(SerializableDate),
	ClearTaskDueDate,
	StopEditingTaskDueDate,

	DragTask { task_id: TaskId, point: Point },
	CancelDragTask,
	PressTask(TaskId),
	LeftClickReleased,

	AnimateProgressbar,
}

impl From<ProjectPageMessage> for Message {
	fn from(value: ProjectPageMessage) -> Self {
		Message::ProjectPageMessage(value)
	}
}

#[derive(Debug)]
pub struct EditTaskState {
	pub task_id: TaskId,
	pub new_name: text_editor::Content,
	pub new_needed_time_minutes: Option<Option<usize>>, // first option is if editing, second is if any time is enterered
	pub edit_due_date: bool,
}

impl EditTaskState {
	pub fn new(task_id: TaskId, new_name: text_editor::Content) -> Self {
		Self {
			task_id,
			new_name,
			new_needed_time_minutes: None,
			edit_due_date: false,
		}
	}
}

#[derive(Debug, Clone)]
pub struct CachedTaskList {
	pub todo: Vec<TaskId>,
	pub done: Vec<TaskId>,
	pub source_code_todo: Vec<TaskId>,
	cache_time: Instant,
}

impl CachedTaskList {
	pub fn new(todo: Vec<TaskId>, done: Vec<TaskId>, source_code_todo: Vec<TaskId>) -> Self {
		Self {
			todo,
			done,
			source_code_todo,
			cache_time: Instant::now(),
		}
	}

	pub fn generate(
		project: &Project,
		task_tag_filter: &HashSet<TaskTagId>,
		search_filter: &Option<String>,
	) -> Self {
		let matches = |task: &Task| {
			task.matches_filter(task_tag_filter)
				&& search_filter
					.as_ref()
					.map(|search_filter| {
						SkimMatcherV2::default()
							.fuzzy_match(&task.name, search_filter)
							.is_some()
					})
					.unwrap_or(true)
		};

		let mut todo_list = Vec::new();
		for (task_id, task) in project.todo_tasks.iter() {
			if matches(task) {
				todo_list.push(task_id);
			}
		}
		let mut done_list = Vec::new();
		for (task_id, task) in project.done_tasks.iter() {
			if matches(task) {
				done_list.push(*task_id);
			}
		}
		let mut source_code_todo_list = Vec::new();
		for (task_id, task) in project.source_code_todos.iter() {
			if matches(task) {
				source_code_todo_list.push(*task_id);
			}
		}
		Self::new(todo_list, done_list, source_code_todo_list)
	}
}

#[derive(Default)]
pub enum ProjectPageAction {
	#[default]
	None,
	Task(iced::Task<Message>),
	OpenManageTaskTagsModal(ProjectId),
	ConfirmDeleteProject{
		project_id: ProjectId,
		project_name: String,
	},
}

impl From<iced::Task<Message>> for ProjectPageAction {
	fn from(value: iced::Task<Message>) -> Self {
		ProjectPageAction::Task(value)
	}
}

#[derive(Debug)]
pub struct ProjectPage {
	pub project_id: ProjectId,
	pub cached_task_list: CachedTaskList,
	edited_project_name: Option<String>,
	pub create_new_task: Option<(String, HashSet<TaskTagId>)>,
	edited_task: Option<EditTaskState>,
	show_done_tasks: bool,
	show_source_code_todos: bool,
	show_color_picker: bool,
	filter_task_tags: HashSet<TaskTagId>,
	search_tasks_filter: Option<String>,
	pressed_task: Option<TaskId>,
	dragged_task: Option<TaskId>,
	start_dragging_point: Option<Point>,
	just_minimal_dragging: bool,
	progressbar_animation: ScalarAnimation,
	previous_project_progress: f32,
	show_context_menu: bool,
}

impl ProjectPage {
	pub fn new(project_id: ProjectId, project: &Project) -> Self {
		let cached_task_list = CachedTaskList::generate(project, &HashSet::new(), &None);

		Self {
			project_id,
			cached_task_list,
			edited_project_name: None,
			create_new_task: None,
			edited_task: None,
			show_done_tasks: false,
			show_source_code_todos: true,
			show_color_picker: false,
			filter_task_tags: HashSet::new(),
			search_tasks_filter: None,
			pressed_task: None,
			dragged_task: None,
			start_dragging_point: None,
			just_minimal_dragging: true,
			progressbar_animation: ScalarAnimation::Idle,
			previous_project_progress: project.get_completion_percentage(),
			show_context_menu: false,
		}
	}

	pub fn update(
		&mut self,
		message: ProjectPageMessage,
		database: &mut Option<Database>,
		preference: &Option<Preferences>,
	) -> ProjectPageAction {
		let command = match message {
			ProjectPageMessage::OpenCreateNewTask => {
				self.create_new_task = Some((String::new(), HashSet::new()));
				let create_new_tasks_at_top = preference
					.as_ref()
					.map(|pref| pref.create_new_tasks_at_top())
					.unwrap_or(true);
				let create_new_task_element_relative_y_offset = if create_new_tasks_at_top {
					0.0
				} else {
					database
						.as_ref()
						.and_then(|db| db.get_project(&self.project_id))
						.map(|project| {
							let mut total_tasks_shown = project.todo_tasks.len();
							if self.show_source_code_todos {
								total_tasks_shown += project.source_code_todos.len()
							}
							if self.show_done_tasks {
								total_tasks_shown += project.done_tasks.len();
							}

							if total_tasks_shown == 0 {
								1.0
							} else {
								project.todo_tasks.len() as f32 / total_tasks_shown as f32
							}
						})
						.unwrap_or(1.0)
				};
				self.edited_task = None;

				iced::Task::batch([
					text_input::focus(CREATE_NEW_TASK_NAME_INPUT_ID.clone()),
					scrollable::snap_to(
						TASK_LIST_ID.clone(),
						RelativeOffset {
							x: 0.0,
							y: create_new_task_element_relative_y_offset,
						},
					)
				])
				.into()
			}
			ProjectPageMessage::CloseCreateNewTask => {
				self.create_new_task = None;
				ProjectPageAction::None
			}
			ProjectPageMessage::ChangeCreateNewTaskName(new_task_name) => {
				if let Some((create_new_task_name, _create_new_task_tags)) =
					&mut self.create_new_task
				{
					*create_new_task_name = new_task_name;
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::ToggleCreateNewTaskTag(tag_id) => {
				if let Some((_create_new_task_name, create_new_task_tags)) =
					&mut self.create_new_task
				{
					if create_new_task_tags.contains(&tag_id) {
						create_new_task_tags.remove(&tag_id);
					} else {
						create_new_task_tags.insert(tag_id);
					}
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::CreateNewTask => {
				if let Some((create_new_task_name, create_new_task_tags)) =
					&mut self.create_new_task
				{
					if let Some(db) = database {
						db.update(DatabaseMessage::CreateTask {
							project_id: self.project_id,
							task_id: generate_task_id(),
							task_name: std::mem::take(create_new_task_name),
							task_tags: std::mem::take(create_new_task_tags),
							create_at_top: preference
								.as_ref()
								.map(|pref| pref.create_new_tasks_at_top())
								.unwrap_or(true),
						});
					}
				}
				self.update(ProjectPageMessage::CloseCreateNewTask, database, preference)
			},

			ProjectPageMessage::ShowContextMenu => { self.show_context_menu = true; ProjectPageAction::None },
			ProjectPageMessage::HideContextMenu => { self.show_context_menu = false; ProjectPageAction::None },
			ProjectPageMessage::OpenManageTaskTagsModal => { self.show_context_menu = false; ProjectPageAction::OpenManageTaskTagsModal(self.project_id) },

			ProjectPageMessage::OpenSearchTasks => {
				self.search_tasks_filter = Some(String::new());
				if let Some(database) = database {
					self.generate_cached_task_list(database);
				}
				text_input::focus(SEARCH_TASKS_TEXT_INPUT_ID.clone()).into()
			}
			ProjectPageMessage::CloseSearchTasks => {
				self.search_tasks_filter = None;
				if let Some(database) = database {
					self.generate_cached_task_list(database);
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::ChangeSearchTasksFilter(new_filter) => {
				self.search_tasks_filter = Some(new_filter);
				if let Some(database) = database {
					self.generate_cached_task_list(database);
				}
				ProjectPageAction::None
			}

			ProjectPageMessage::ImportSourceCodeTodosDialog => {
				self.show_context_menu = false;
				iced::Task::perform(Self::pick_todo_source_folders_dialog(), |folders| {
					if let Some(folders) = folders {
						ProjectPageMessage::ImportSourceCodeTodos(folders).into()
					} else {
						ProjectPageMessage::ImportSourceCodeTodosDialogCanceled.into()
					}
				})
				.into()
			}
			ProjectPageMessage::ImportSourceCodeTodosDialogCanceled => ProjectPageAction::None,
			ProjectPageMessage::ImportSourceCodeTodos(todos) => {
				if let Some(database) = database {
					database.modify(|projects| {
						if let Some(project) = projects.get_mut(&self.project_id) {
							project.source_code_todos.clear();
							for task in todos {
								project.source_code_todos.insert(generate_task_id(), task);
							}
						}
					})
				}
				ProjectPageAction::None
			}

			ProjectPageMessage::ShowSourceCodeTodos(show) => {
				self.show_source_code_todos = show;
				ProjectPageAction::None
			}

			ProjectPageMessage::ShowDoneTasks(show) => {
				self.show_done_tasks = show;
				ProjectPageAction::None
			}

			ProjectPageMessage::ToggleFilterTaskTag(task_tag_id) => {
				if self.filter_task_tags.contains(&task_tag_id) {
					self.filter_task_tags.remove(&task_tag_id);
				} else {
					self.filter_task_tags.insert(task_tag_id);
				}
				if let Some(database) = database {
					self.generate_cached_task_list(database);
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::UnsetFilterTaskTag(task_tag_id) => {
				self.filter_task_tags.remove(&task_tag_id);
				if let Some(database) = database {
					self.generate_cached_task_list(database);
				}
				ProjectPageAction::None
			}

			ProjectPageMessage::ShowColorPicker => {
				self.show_color_picker = true;
				ProjectPageAction::None
			}
			ProjectPageMessage::HideColorPicker => {
				self.show_color_picker = false;
				ProjectPageAction::None
			}
			ProjectPageMessage::ChangeProjectColor(new_color) => {
				self.show_color_picker = false;
				if let Some(database) = database {
					database.modify(|projects| {
						if let Some(project) = projects.get_mut(&self.project_id) {
							project.color = new_color.into();
						}
					});
				}
				ProjectPageAction::None
			}

			ProjectPageMessage::EditProjectName => {
				let project_name = database
					.as_ref()
					.and_then(|db| {
						db.get_project(&self.project_id)
							.map(|project| project.name.clone())
					})
					.unwrap_or_default();
				self.edited_project_name = Some(project_name);
				text_input::focus(PROJECT_NAME_TEXT_INPUT_ID.clone()).into()
			}
			ProjectPageMessage::ChangeEditedProjectName(edited_name) => {
				self.edited_project_name = Some(edited_name);
				ProjectPageAction::None
			}
			ProjectPageMessage::StopEditingProjectName => {
				self.edited_project_name = None;
				ProjectPageAction::None
			}
			ProjectPageMessage::ChangeProjectName => {
				if let Some(db) = database {
					if let Some(edited_project_name) = &mut self.edited_project_name {
						db.update(DatabaseMessage::ChangeProjectName {
							project_id: self.project_id,
							new_name: std::mem::take(edited_project_name),
						});
						return self.update(
							ProjectPageMessage::StopEditingProjectName,
							database,
							preference,
						);
					}
				}
				self.update(
					ProjectPageMessage::StopEditingProjectName,
					database,
					preference,
				)
			},
			ProjectPageMessage::ConfirmDeleteProject => {
				self.show_context_menu = false;

				let project_name = database.as_ref().and_then(|db|
					db.get_project(&self.project_id).map(|project| project.name.clone())
				)
				.unwrap_or("<invalid project id>".to_string());

				ProjectPageAction::ConfirmDeleteProject {
					project_id: self.project_id,
					project_name,
				}
			}

			ProjectPageMessage::EditTask(task_id) => {
				let task_name = database
					.as_ref()
					.and_then(|db| {
						db.get_task(&self.project_id, &task_id)
							.map(|task| task.name.clone())
					})
					.unwrap_or_default();
				self.edited_task = Some(EditTaskState::new(
					task_id,
					text_editor::Content::with_text(&task_name),
				));
				self.update(ProjectPageMessage::CloseCreateNewTask, database, preference)
			}
			ProjectPageMessage::StopEditingTask => {
				self.edited_task = None;
				ProjectPageAction::None
			}
			ProjectPageMessage::TaskNameAction(action) => {
				if let Some(edit_task_state) = &mut self.edited_task {
					let is_edit = action.is_edit();
					edit_task_state.new_name.perform(action);
					if is_edit {
						if let Some(database) = database {
							database.update(DatabaseMessage::ChangeTaskName {
								project_id: self.project_id,
								task_id: edit_task_state.task_id,
								new_task_name: edit_task_state.new_name.text(),
							});
						}
					}
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::ToggleTaskTag(task_tag_id) => {
				if let Some(edit_task_state) = &mut self.edited_task {
					if let Some(database) = database {
						database.update(DatabaseMessage::ToggleTaskTag {
							project_id: self.project_id,
							task_id: edit_task_state.task_id,
							task_tag_id,
						});
					}
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::EditTaskNeededTime => {
				if let Some(edit_task_state) = &mut self.edited_task {
					let previous_task_needed_minutes = database.as_ref().and_then(|db| {
						db.get_task(&self.project_id, &edit_task_state.task_id)
							.and_then(|task| task.needed_time_minutes)
					});

					edit_task_state.new_needed_time_minutes = Some(previous_task_needed_minutes);
				}
				text_input::focus(EDIT_NEEDED_TIME_TEXT_INPUT_ID.clone()).into()
			}
			ProjectPageMessage::ClearTaskNeededTime => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.new_needed_time_minutes = None;
					if let Some(database) = database {
						database.update(DatabaseMessage::ChangeTaskNeededTime {
							project_id: self.project_id,
							task_id: edit_task_state.task_id,
							new_needed_time_minutes: None,
						});
					}
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::ChangeNewTaskNeededTimeInput(new_needed_time_minutes) => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.new_needed_time_minutes = Some(new_needed_time_minutes);
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::InvalidNeededTimeInput => ProjectPageAction::None,
			ProjectPageMessage::ChangeTaskNeededTime => {
				if let Some(edit_task_state) = &mut self.edited_task {
					if let Some(new_needed_time_minutes) = edit_task_state.new_needed_time_minutes {
						if let Some(database) = database {
							database.modify(|projects| {
								if let Some(project) = projects.get_mut(&self.project_id) {
									if let Some(task) =
										project.get_task_mut(&edit_task_state.task_id)
									{
										task.needed_time_minutes = new_needed_time_minutes;
									}
								}
							});
						}
					}
					edit_task_state.new_needed_time_minutes = None;
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::EditTaskDueDate => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_due_date = true;
				}
				text_input::focus(EDIT_DUE_DATE_TEXT_INPUT_ID.clone()).into()
			}
			ProjectPageMessage::ChangeTaskDueDate(new_due_date) => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_due_date = false;
					if let Some(database) = database {
						database.update(DatabaseMessage::ChangeTaskDueDate {
							project_id: self.project_id,
							task_id: edit_task_state.task_id,
							new_due_date: new_due_date.into(),
						});
					}
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::ClearTaskDueDate => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_due_date = false;
					if let Some(database) = database {
						database.update(DatabaseMessage::ChangeTaskDueDate {
							project_id: self.project_id,
							task_id: edit_task_state.task_id,
							new_due_date: None,
						});
					}
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::StopEditingTaskDueDate => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_due_date = false;
				}
				ProjectPageAction::None
			}

			ProjectPageMessage::DragTask { task_id, point } => {
				self.dragged_task = Some(task_id);
				if let Some(start_dragging_point) = self.start_dragging_point {
					if self.just_minimal_dragging {
						self.just_minimal_dragging =
							start_dragging_point.distance(point) < MINIMAL_DRAG_DISTANCE;
					}
				} else {
					self.start_dragging_point = Some(point);
					self.just_minimal_dragging = true;
				}
				ProjectPageAction::None
			}
			ProjectPageMessage::CancelDragTask => {
				self.dragged_task = None;
				self.start_dragging_point = None;
				self.just_minimal_dragging = true;
				ProjectPageAction::None
			}
			ProjectPageMessage::PressTask(task_id) => {
				self.pressed_task = Some(task_id);
				ProjectPageAction::None
			}
			ProjectPageMessage::LeftClickReleased => {
				let action = if self.just_minimal_dragging {
					if let Some(pressed_task) = &self.pressed_task {
						self.update(
							ProjectPageMessage::EditTask(*pressed_task),
							database,
							preference,
						)
					} else {
						ProjectPageAction::None
					}
				} else {
					ProjectPageAction::None
				};
				self.pressed_task = None;
				self.dragged_task = None;
				self.start_dragging_point = None;
				self.just_minimal_dragging = true;
				action
			}

			ProjectPageMessage::AnimateProgressbar => {
				self.progressbar_animation.update();
				ProjectPageAction::None
			}
		};

		if let Some(database_ref) = database {
			if self.cached_task_list.cache_time < *database_ref.last_changed_time() {
				self.generate_cached_task_list(database_ref);
			}

			if let Some(project) = database_ref.get_project(&self.project_id) {
				let new_project_progress = project.get_completion_percentage();
				if new_project_progress != self.previous_project_progress {
					self.start_progressbar_animation(
						self.previous_project_progress,
						new_project_progress,
					);
					self.previous_project_progress = new_project_progress;
				}
			}
		}

		command
	}

	pub fn subscription(&self) -> Subscription<ProjectPageMessage> {
		Subscription::batch([
			self.progressbar_animation
				.subscription()
				.map(|_| ProjectPageMessage::AnimateProgressbar),
			iced::event::listen_with(move |event, _status, _id| match event {
				Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
					Some(ProjectPageMessage::LeftClickReleased)
				}
				_ => None,
			}),
			keyboard::on_key_press(|key, modifiers| match key.as_ref() {
				keyboard::Key::Character("r") if modifiers.command() => {
					Some(ProjectPageMessage::EditProjectName)
				}
				keyboard::Key::Character("f") if modifiers.command() => {
					Some(ProjectPageMessage::OpenSearchTasks)
				}
				keyboard::Key::Character("n") if modifiers.command() && !modifiers.shift() => {
					Some(ProjectPageMessage::OpenCreateNewTask)
				}
				_ => None,
			}),
		])
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, Message> {
		if let Some(database) = &app.database {
			if let Some(project) = database.get_project(&self.project_id) {
				column![
					self.project_details_view(project),
					task_list(
						self.project_id,
						project,
						&self.cached_task_list,
						&self.edited_task,
						self.dragged_task,
						self.just_minimal_dragging,
						app.sidebar_page.task_dropzone_hovered,
						self.show_done_tasks,
						self.show_source_code_todos,
						&self.create_new_task,
						&app.stopwatch_page,
						app.preferences
							.as_ref()
							.map(|pref| pref.date_formatting())
							.unwrap_or_default(),
						app.preferences
							.as_ref()
							.map(|pref| pref.create_new_tasks_at_top())
							.unwrap_or(true),
					),
				]
				// .spacing(SPACING_AMOUNT) this is not needed since every task in the list has a SPACING_AMOUNT height dropzone
				.width(Fill)
				.height(Fill)
				.into()
			} else {
				text("<Invalid ProjectId>").into()
			}
		} else {
			column![].into()
		}
	}

	fn project_details_view<'a>(&'a self, project: &'a Project) -> Element<'a, Message> {
		let project_name: Element<Message> =
			if let Some(edited_project_name) = &self.edited_project_name {
				unfocusable(
					text_input("New project name", edited_project_name)
						.id(PROJECT_NAME_TEXT_INPUT_ID.clone())
						.size(TITLE_TEXT_SIZE)
						.on_input(|edited_name| {
							ProjectPageMessage::ChangeEditedProjectName(edited_name).into()
						})
						.on_submit(ProjectPageMessage::ChangeProjectName.into())
						.style(text_input_style_default),
					ProjectPageMessage::StopEditingProjectName.into(),
				)
				.into()
			} else {
				row![
					text(&project.name).size(TITLE_TEXT_SIZE),
					edit_project_name_button(),
				]
				.spacing(SPACING_AMOUNT)
				.align_y(Alignment::Center)
				.into()
			};

		let show_color_picker_button = edit_color_palette_button(
			project.color.into(),
			self.show_color_picker,
			if self.show_color_picker {
				ProjectPageMessage::HideColorPicker.into()
			} else {
				ProjectPageMessage::ShowColorPicker.into()
			},
		);

		let color_picker = DropDown::new(
			show_color_picker_button,
			color_palette(project.color.into(), |c| {
				ProjectPageMessage::ChangeProjectColor(c).into()
			}),
			self.show_color_picker
		)
		.width(Fill)
		.alignment(drop_down::Alignment::End)
		.on_dismiss(ProjectPageMessage::HideColorPicker.into());

		let mut task_tags_list: Vec<Element<Message>> = Vec::new();
		for (tag_id, tag) in project.task_tags.iter() {
			task_tags_list.push(
				task_tag_button(tag, self.filter_task_tags.contains(&tag_id), true, true)
					.on_press(ProjectPageMessage::ToggleFilterTaskTag(tag_id).into())
					.into(),
			);
		}

		let search_tasks_element: Element<Message> = if let Some(search_tasks_filter) =
			&self.search_tasks_filter
		{
			row![
				unfocusable(
					text_input("Search tasks...", search_tasks_filter)
						.id(SEARCH_TASKS_TEXT_INPUT_ID.clone())
						.icon(text_input::Icon {
							font: BOOTSTRAP_FONT,
							code_point: icon_to_char(Bootstrap::Search),
							size: None,
							spacing: SMALL_SPACING_AMOUNT as f32,
							side: text_input::Side::Left,
						})
						.on_input(|new_search_filter| {
							ProjectPageMessage::ChangeSearchTasksFilter(new_search_filter).into()
						})
						.style(text_input_style_only_round_left),
					ProjectPageMessage::CloseSearchTasks.into()
				),
				cancel_search_tasks_button(),
			]
			.padding(Padding {
				left: LARGE_PADDING_AMOUNT + LARGE_PADDING_AMOUNT,
				..Padding::ZERO
			})
			.into()
		} else {
			search_tasks_button().into()
		};

		let quick_actions: Element<Message> = row![
			search_tasks_element,
			project_context_menu_button(self.show_context_menu),
		]
		.spacing(SPACING_AMOUNT)
		.into();

		let spacer = || Space::new(Fill, SPACING_AMOUNT);

		column![
			column![row![
				color_picker,
				project_name,
				if self.edited_project_name.is_some() {
					quick_actions
				} else {
					container(quick_actions)
						.width(Fill)
						.align_x(Horizontal::Right)
						.into()
				}
			]
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center)]
			.width(Fill),
			spacer(),
			row![
				container("Tags:").padding(HORIZONTAL_SCROLLABLE_PADDING),
				horizontal_scrollable(Row::with_children(task_tags_list).spacing(SPACING_AMOUNT))
					.width(Fill),
			]
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center),
			row![
				text(format!(
					"{}/{} finished ({}%)",
					project.done_tasks.len(),
					project.total_tasks(),
					(project.get_completion_percentage() * 100.0).round()
				))
				.width(Fill),
				container(create_new_task_button(self.create_new_task.is_none()))
					.width(Fill)
					.align_x(Horizontal::Right),
			]
			.width(Fill)
			.align_y(Alignment::Center),
			spacer(),
			completion_bar(
				self.progressbar_animation
					.get_value()
					.unwrap_or(self.previous_project_progress)
			),
		]
		.padding(Padding {
			top: PADDING_AMOUNT,
			bottom: 0.0, // task_list already has padding on the top due to dropzone padding/spacing
			left: PADDING_AMOUNT,
			right: PADDING_AMOUNT,
		})
		.into()
	}

	fn start_progressbar_animation(&mut self, start_percentage: f32, target_percentage: f32) {
		self.progressbar_animation =
			ScalarAnimation::start(start_percentage, target_percentage, 0.125);
	}

	fn generate_cached_task_list(&mut self, database: &Database) {
		if let Some(project) = database.get_project(&self.project_id) {
			self.cached_task_list =
				CachedTaskList::generate(project, &self.filter_task_tags, &self.search_tasks_filter)
		}
	}

	async fn pick_todo_source_folders_dialog() -> Option<Vec<Task>> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Import Todos from source files")
			.pick_folders()
			.await;

		file_dialog_result.map(|folder_handles| {
			let mut todos = Vec::new();

			for folder_handle in folder_handles {
				let folder_path = folder_handle.path();
				if folder_path
					.file_name()
					.and_then(|file_name| {
						file_name
							.to_str()
							.map(|file_name| file_name.starts_with('.'))
					})
					.unwrap_or(false)
				{
					continue;
				}
				for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
					if entry
						.file_name()
						.to_str()
						.map(|file_name| file_name.starts_with('.'))
						.unwrap_or(false)
					{
						continue;
					}
					if entry.metadata().map(|meta| meta.is_dir()).unwrap_or(false) {
						continue;
					}
					if let Ok(file) = File::open(entry.path()) {
						for (i, line) in io::BufReader::new(file)
							.lines()
							.map_while(Result::ok)
							.enumerate()
						{
							let mut search_todo = |keyword: &'static str| {
								if let Some(index) =
									line.to_lowercase().find(&keyword.to_lowercase())
								{
									let mut string_quotes_counter = 0;
									for c in line[0..index].chars() {
										if c == '\"' || c == '\'' {
											string_quotes_counter += 1;
										}
									}

									if string_quotes_counter % 2 == 0 {
										let line = line[index + keyword.len()..].to_string();
										let line = line.strip_prefix(':').unwrap_or(&line);
										let line = line.strip_prefix(' ').unwrap_or(line);
										let source = entry.path().display();
										let line_number = i + 1;
										todos.push(Task::new(
											format!("{line}:\n    {source} on line {line_number}"),
											HashSet::new(),
										));
									}
								}
							};

							// case insensitive!
							search_todo("// todo");
							search_todo("//todo");
							search_todo("# todo");
							search_todo("#todo");
						}
					}
				}
			}

			todos
		})
	}
}
