use std::{collections::HashSet, fs::File, io::{self, BufRead}, time::Instant};
use iced::{alignment::{Alignment, Horizontal}, theme, widget::{button, column, container, row, scrollable, scrollable::RelativeOffset, text, text_editor, text_input, Row}, Color, Command, Element, Length, Padding, Point};
use iced_aw::BOOTSTRAP_FONT;
use once_cell::sync::Lazy;
use walkdir::WalkDir;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use crate::{
	components::{cancel_search_tasks_button, color_palette, color_palette_item_button, completion_bar, create_new_task_button, delete_project_button, horizontal_scrollable, import_source_code_todos_button, manage_task_tags_button, search_tasks_button, task_list, task_tag_button, unfocusable, CREATE_NEW_TASK_NAME_INPUT_ID, EDIT_DUE_DATE_TEXT_INPUT_ID, EDIT_NEEDED_TIME_TEXT_INPUT_ID, TASK_LIST_ID},
	core::{generate_task_id, Database, DatabaseMessage, Project, ProjectId, SerializableDate, Task, TaskId, TaskTagId},
	project_tracker::{ProjectTrackerApp, UiMessage},
	styles::{HiddenSecondaryButtonStyle, TextInputStyle, LARGE_PADDING_AMOUNT, MINIMAL_DRAG_DISTANCE, PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TINY_SPACING_AMOUNT, TITLE_TEXT_SIZE},
};

static PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static SEARCH_TASKS_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum ProjectPageMessage {
	OpenCreateNewTask,
	CloseCreateNewTask,
	ChangeCreateNewTaskName(String),
	ToggleCreateNewTaskTag(TaskTagId),
	CreateNewTask,

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

	EditProjectName,
	StopEditingProjectName,
	ChangeEditedProjectName(String),
	ChangeProjectName,

	EditTask(TaskId),
	StopEditingTask,
	TaskNameAction(text_editor::Action),
	ToggleTaskTag(TaskTagId),
	EditTaskNeededTime,
	ClearTaskNeededTime,
	InvalidNeededTimeInput,
	StopEditingTaskNeededTime,
	EditTaskDueDate,
	ChangeTaskDueDate(SerializableDate),
	ClearTaskDueDate,
	StopEditingTaskDueDate,

	DragTask{
		task_id: TaskId,
		point: Point,
	},
	CancelDragTask,
	PressTask(TaskId),
	LeftClickReleased,
}

impl From<ProjectPageMessage> for UiMessage {
	fn from(value: ProjectPageMessage) -> Self {
		UiMessage::ProjectPageMessage(value)
	}
}

#[derive(Debug)]
pub struct EditTaskState {
	pub task_id: TaskId,
	pub new_name: text_editor::Content,
	pub edit_needed_time: bool,
	pub edit_due_date: bool,
}

impl EditTaskState {
	pub fn new(task_id: TaskId, new_name: text_editor::Content) -> Self {
		Self {
			task_id,
			new_name,
			edit_needed_time: false,
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

	pub fn generate(project: &Project, task_tag_filter: &HashSet<TaskTagId>, search_filter: &Option<String>) -> Self {
		let matches = |task: &Task| {
			task.matches_filter(task_tag_filter) &&
			search_filter
				.as_ref()
				.map(|search_filter|
					SkimMatcherV2::default()
						.fuzzy_match(&task.name, search_filter)
						.is_some()
				)
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
		}
	}
}

impl ProjectPage {
	pub fn update(&mut self, message: ProjectPageMessage, database: &mut Option<Database>) -> Command<UiMessage> {
		let command = match message {
			ProjectPageMessage::OpenCreateNewTask => {
				self.create_new_task = Some((String::new(), HashSet::new()));
				let create_new_task_element_relative_y_offset = if self.show_done_tasks {
					database
						.as_ref()
						.and_then(|db| db.projects().get(&self.project_id))
						.map(|project| {
							if project.total_tasks() == 0 {
								1.0
							}
							else {
								project.tasks_todo() as f32 / project.total_tasks() as f32
							}
						})
						.unwrap_or(1.0)
				}
				else {
					1.0
				};
				Command::batch([
					text_input::focus(CREATE_NEW_TASK_NAME_INPUT_ID.clone()),
					scrollable::snap_to(TASK_LIST_ID.clone(), RelativeOffset{ x: 0.0, y: create_new_task_element_relative_y_offset }),
					self.update(ProjectPageMessage::StopEditingTask, database),
				])
			},
			ProjectPageMessage::CloseCreateNewTask => { self.create_new_task = None; Command::none() },
			ProjectPageMessage::ChangeCreateNewTaskName(new_task_name) => {
				if let Some((create_new_task_name, _create_new_task_tags)) = &mut self.create_new_task {
					*create_new_task_name = new_task_name;
				}
				Command::none()
			},
			ProjectPageMessage::ToggleCreateNewTaskTag(tag_id) => {
				if let Some((_create_new_task_name, create_new_task_tags)) = &mut self.create_new_task {
					if create_new_task_tags.contains(&tag_id) {
						create_new_task_tags.remove(&tag_id);
					}
					else {
						create_new_task_tags.insert(tag_id);
					}
				}
				Command::none()
			},
			ProjectPageMessage::CreateNewTask => {
				if let Some((create_new_task_name, create_new_task_tags)) = &mut self.create_new_task {
					if let Some(db) = database {
						return Command::batch([
							db.update(DatabaseMessage::CreateTask {
								project_id: self.project_id,
								task_id: generate_task_id(),
								task_name: std::mem::take(create_new_task_name),
								task_tags: std::mem::take(create_new_task_tags),
							}),
							self.update(ProjectPageMessage::CloseCreateNewTask, database)
						]);
					}
				}
				self.update(ProjectPageMessage::CloseCreateNewTask, database)
			},

			ProjectPageMessage::OpenSearchTasks => {
				self.search_tasks_filter = Some(String::new());
				text_input::focus(SEARCH_TASKS_TEXT_INPUT_ID.clone())
			},
			ProjectPageMessage::CloseSearchTasks => { self.search_tasks_filter = None; Command::none() },
			ProjectPageMessage::ChangeSearchTasksFilter(new_filter) => {
				self.search_tasks_filter = Some(new_filter);
				if let Some(database) = database {
					self.generate_cached_task_list(database);
				}
				Command::none()
			},

			ProjectPageMessage::ImportSourceCodeTodosDialog => Command::perform(
				Self::pick_todo_source_folders_dialog(),
				|folders| {
					if let Some(folders) = folders {
						ProjectPageMessage::ImportSourceCodeTodos(folders).into()
					}
					else {
						ProjectPageMessage::ImportSourceCodeTodosDialogCanceled.into()
					}
				}),
			ProjectPageMessage::ImportSourceCodeTodosDialogCanceled => Command::none(),
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
				Command::none()
			},

			ProjectPageMessage::ShowSourceCodeTodos(show) => { self.show_source_code_todos = show; Command::none() },

			ProjectPageMessage::ShowDoneTasks(show) => { self.show_done_tasks = show; Command::none() },

			ProjectPageMessage::ToggleFilterTaskTag(task_tag_id) => {
				if self.filter_task_tags.contains(&task_tag_id) {
					self.filter_task_tags.remove(&task_tag_id);
				}
				else {
					self.filter_task_tags.insert(task_tag_id);
				}
				if let Some(database) = database {
					self.generate_cached_task_list(database);
				}
				Command::none()
			},
			ProjectPageMessage::UnsetFilterTaskTag(task_tag_id) => {
				self.filter_task_tags.remove(&task_tag_id);
				if let Some(database) = database {
					self.generate_cached_task_list(database);
				}
				Command::none()
			},

			ProjectPageMessage::ShowColorPicker => { self.show_color_picker = true; Command::none() },
			ProjectPageMessage::HideColorPicker => { self.show_color_picker = false; Command::none() },

			ProjectPageMessage::EditProjectName => {
				let project_name = database.as_ref()
					.and_then(|db|
						db.projects()
							.get(&self.project_id)
							.map(|project| project.name.clone())
					)
					.unwrap_or_default();
				self.edited_project_name = Some(project_name);
				text_input::focus(PROJECT_NAME_TEXT_INPUT_ID.clone())
			},
			ProjectPageMessage::ChangeEditedProjectName(edited_name) => { self.edited_project_name = Some(edited_name); Command::none() },
			ProjectPageMessage::StopEditingProjectName => { self.edited_project_name = None; Command::none() },
			ProjectPageMessage::ChangeProjectName => {
				if let Some(db) = database {
					if let Some(edited_project_name) = &mut self.edited_project_name {
						return Command::batch([
							db.update(DatabaseMessage::ChangeProjectName {
								project_id: self.project_id,
								new_name: std::mem::take(edited_project_name)
							}),
							self.update(ProjectPageMessage::StopEditingProjectName, database)
						]);
					}
				}
				self.update(ProjectPageMessage::StopEditingProjectName, database)
			},

			ProjectPageMessage::EditTask(task_id) => {
				let task_name = database.as_ref().and_then(|db| {
					db.projects().get(&self.project_id)
						.and_then(|project|
							project.get_task(&task_id)
								.map(|task| task.name.clone())
						)
				}).unwrap_or_default();
				self.edited_task = Some(EditTaskState::new(task_id, text_editor::Content::with_text(&task_name)));
				self.update(ProjectPageMessage::CloseCreateNewTask, database)
			},
			ProjectPageMessage::StopEditingTask => { self.edited_task = None; Command::none() },
			ProjectPageMessage::TaskNameAction(action) => {
				if let Some(edit_task_state) = &mut self.edited_task {
					let is_edit = action.is_edit();
					edit_task_state.new_name.perform(action);
					if is_edit {
						if let Some(database) = database {
							return database.update(DatabaseMessage::ChangeTaskName {
								project_id: self.project_id,
								task_id: edit_task_state.task_id,
								new_task_name: edit_task_state.new_name.text()
							});
						}
					}
				}
				Command::none()
			},
			ProjectPageMessage::ToggleTaskTag(task_tag_id) => {
				if let Some(edit_task_state) = &mut self.edited_task {
					if let Some(database) = database {
						return database.update(DatabaseMessage::ToggleTaskTag {
							project_id: self.project_id,
							task_id: edit_task_state.task_id,
							task_tag_id
						});
					}
				}
				Command::none()
			},
			ProjectPageMessage::EditTaskNeededTime => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_needed_time = true;
				}
				text_input::focus(EDIT_NEEDED_TIME_TEXT_INPUT_ID.clone())
			},
			ProjectPageMessage::ClearTaskNeededTime => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_needed_time = false;
					if let Some(database) = database {
						return database.update(DatabaseMessage::ChangeTaskNeededTime {
							project_id: self.project_id,
							task_id: edit_task_state.task_id,
							new_needed_time_minutes: None,
						});
					}
				}
				Command::none()
			},
			ProjectPageMessage::InvalidNeededTimeInput => Command::none(),
			ProjectPageMessage::StopEditingTaskNeededTime => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_needed_time = false;
				}
				Command::none()
			},
			ProjectPageMessage::EditTaskDueDate => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_due_date = true;
				}
				text_input::focus(EDIT_DUE_DATE_TEXT_INPUT_ID.clone())
			},
			ProjectPageMessage::ChangeTaskDueDate(new_due_date) => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_due_date = false;
					if let Some(database) = database {
						return database.update(DatabaseMessage::ChangeTaskDueDate {
							project_id: self.project_id,
							task_id: edit_task_state.task_id,
							new_due_date: new_due_date.into()
						});
					}
				}
				Command::none()
			},
			ProjectPageMessage::ClearTaskDueDate => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_due_date = false;
					if let Some(database) = database {
						return database.update(DatabaseMessage::ChangeTaskDueDate {
							project_id: self.project_id,
							task_id: edit_task_state.task_id,
							new_due_date: None,
						});
					}
				}
				Command::none()
			},
			ProjectPageMessage::StopEditingTaskDueDate => {
				if let Some(edit_task_state) = &mut self.edited_task {
					edit_task_state.edit_due_date = false;
				}
				Command::none()
			},

			ProjectPageMessage::DragTask{ task_id, point } => {
				self.dragged_task = Some(task_id);
				if let Some(start_dragging_point) = self.start_dragging_point {
					if self.just_minimal_dragging {
						self.just_minimal_dragging = start_dragging_point.distance(point) < MINIMAL_DRAG_DISTANCE;
					}
				}
				else {
					self.start_dragging_point = Some(point);
					self.just_minimal_dragging = true;
				}
				Command::none()
			},
			ProjectPageMessage::CancelDragTask => {
				self.dragged_task = None;
				self.start_dragging_point = None;
				self.just_minimal_dragging = true;
				Command::none()
			},
			ProjectPageMessage::PressTask(task_id) => {
				self.pressed_task = Some(task_id);
				Command::none()
			},
			ProjectPageMessage::LeftClickReleased => {
				let command = if self.just_minimal_dragging {
					if let Some(pressed_task) = &self.pressed_task {
						self.update(ProjectPageMessage::EditTask(*pressed_task), database)
					}
					else {
						Command::none()
					}
				}
				else {
					Command::none()
				};
				self.pressed_task = None;
				self.dragged_task = None;
				self.start_dragging_point = None;
				self.just_minimal_dragging = true;
				command
			},
		};

		if let Some(database) = database {
			if self.cached_task_list.cache_time < *database.last_changed_time() {
				self.generate_cached_task_list(database);
			}
		}

		command
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		if let Some(database) = &app.database {
			if let Some(project) = database.projects().get(&self.project_id) {
				column![
					self.project_details_view(self.project_id, project),

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
						app.preferences.as_ref().map(|pref| pref.date_formatting()).unwrap_or_default()
					),
				]
				// .spacing(SPACING_AMOUNT) this is not needed since every task in the list has a SPACING_AMOUNT height dropzone
				.width(Length::Fill)
				.height(Length::Fill)
				.into()
			}
			else {
				text("<Invalid ProjectId>").into()
			}
		}
		else {
			column![].into()
		}
	}

	fn project_details_view(&self, project_id: ProjectId, project: &Project) -> Element<UiMessage> {
		let project_name : Element<UiMessage> = if let Some(edited_project_name) = &self.edited_project_name {
			unfocusable(
				text_input("New project name", edited_project_name)
					.id(PROJECT_NAME_TEXT_INPUT_ID.clone())
					.size(TITLE_TEXT_SIZE)
					.on_input(|edited_name| ProjectPageMessage::ChangeEditedProjectName(edited_name).into())
					.on_submit(ProjectPageMessage::ChangeProjectName.into())
					.style(theme::TextInput::Custom(Box::new(TextInputStyle::default()))),

				ProjectPageMessage::StopEditingProjectName.into()
			)
			.into()
		}
		else {
			button(
				text(&project.name).size(TITLE_TEXT_SIZE)
			)
			.on_press(ProjectPageMessage::EditProjectName.into())
			.style(theme::Button::custom(HiddenSecondaryButtonStyle))
			.into()
		};

		let show_color_picker_button = color_palette_item_button(
			project.color.into(),
			false,
			if self.show_color_picker {
				ProjectPageMessage::HideColorPicker.into()
			}
			else {
				ProjectPageMessage::ShowColorPicker.into()
			}
		);

		let mut task_tags_list: Vec<Element<UiMessage>> = Vec::new();
		for (tag_id, tag) in project.task_tags.iter() {
			task_tags_list.push(
				task_tag_button(tag, self.filter_task_tags.contains(&tag_id), true, true)
					.on_press(ProjectPageMessage::ToggleFilterTaskTag(tag_id).into())
					.into()
			);
		}

		let search_tasks_element: Element<UiMessage> = if let Some(search_tasks_filter) = &self.search_tasks_filter {
			row![
				unfocusable(
					text_input(
						"Search tasks...",
						search_tasks_filter
					)
					.id(SEARCH_TASKS_TEXT_INPUT_ID.clone())
					.icon(text_input::Icon {
						font: BOOTSTRAP_FONT,
						code_point: '\u{F52A}', // Search icon, see Bootstrap::Search
						size: None,
						spacing: SMALL_SPACING_AMOUNT as f32,
						side: text_input::Side::Left,
					})
					.on_input(|new_search_filter| ProjectPageMessage::ChangeSearchTasksFilter(new_search_filter).into())
					.style(theme::TextInput::Custom(Box::new(TextInputStyle::ONLY_ROUND_LEFT))),

					ProjectPageMessage::CloseSearchTasks.into()
				),
				cancel_search_tasks_button(),
			]
			.padding(Padding {
				left: LARGE_PADDING_AMOUNT + LARGE_PADDING_AMOUNT,
				..Padding::ZERO
			})
			.into()
		}
		else {
			search_tasks_button().into()
		};

		let delete_project_button_element: Element<UiMessage> = delete_project_button(self.project_id, &project.name).into();

		let quick_actions: Element<UiMessage> = row![
			search_tasks_element,
			import_source_code_todos_button(),
			delete_project_button_element,
		]
		.spacing(SPACING_AMOUNT)
		.into();

		column![
			column![
				row![
					show_color_picker_button,
					project_name,
					if self.edited_project_name.is_some() {
						quick_actions
					}
					else {
						container(
							quick_actions
						)
						.width(Length::Fill)
						.align_x(Horizontal::Right)
						.into()
					}
				]
				.spacing(TINY_SPACING_AMOUNT)
				.align_items(Alignment::Center)
			]
			.push_maybe(if self.show_color_picker {
				Some(color_palette(project.color.into(), move |c: Color| DatabaseMessage::ChangeProjectColor{ project_id, new_color: c.into() }.into()))
			}
			else {
				None
			})
			.width(Length::Fill),


			row![
				text("Tags:"),

				horizontal_scrollable(
					Row::with_children(task_tags_list)
						.spacing(SPACING_AMOUNT)
				)
				.width(Length::Fill),

				manage_task_tags_button(self.project_id),
			]
			.spacing(SPACING_AMOUNT)
			.align_items(Alignment::Center),

			row![
				text(
					format!(
						"{}/{} finished ({}%)",
						project.tasks_done(),
						project.total_tasks(),
						(project.get_completion_percentage() * 100.0).round()
					)
				)
				.width(Length::Fill),

				container(create_new_task_button(self.create_new_task.is_none()))
					.width(Length::Fill)
					.align_x(Horizontal::Right),
			]
			.width(Length::Fill)
			.align_items(Alignment::Center),

			completion_bar(project.get_completion_percentage()),
		]
		.padding(Padding {
			top: PADDING_AMOUNT,
			bottom: 0.0, // task_list already has padding on the top due to dropzone padding/spacing
			left: PADDING_AMOUNT,
			right: PADDING_AMOUNT,
		})
		.spacing(SPACING_AMOUNT)
		.into()
	}

	fn generate_cached_task_list(&mut self, database: &Database) {
		if let Some(project) = database.projects().get(&self.project_id) {
			self.cached_task_list = CachedTaskList::generate(project, &self.filter_task_tags, &self.search_tasks_filter)
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
				if folder_path.file_name().and_then(|file_name| file_name.to_str().map(|file_name| file_name.starts_with('.'))).unwrap_or(false) {
					continue;
				}
				for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
					if entry.file_name().to_str().map(|file_name| file_name.starts_with('.')).unwrap_or(false) {
						continue;
					}
					if entry.metadata().map(|meta| meta.is_dir()).unwrap_or(false) {
						continue;
					}
					if let Ok(file) = File::open(entry.path()) {
						for (i, line) in io::BufReader::new(file).lines().map_while(Result::ok).enumerate() {
							let mut search_todo = |keyword: &'static str| {
								if let Some(index) = line.to_lowercase().find(&keyword.to_lowercase()) {
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
											HashSet::new()
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
