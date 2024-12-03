use crate::{
	components::{
		cancel_search_tasks_button, color_palette, completion_bar, edit_color_palette_button, horizontal_scrollable, open_create_task_modal_button, project_context_menu_button, search_tasks_button, sort_dropdown_button, task_list, task_tag_button, unfocusable, ScalarAnimation, HORIZONTAL_SCROLLABLE_PADDING
	}, core::{IcedColorConversion, SortModeUI}, icons::{icon_to_char, Bootstrap, BOOTSTRAP_FONT}, pages::{ContentPageAction, ContentPageMessage}, project_tracker::{Message, ProjectTrackerApp}, styles::{
		text_input_style_borderless, text_input_style_only_round_left, PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE
	}, OptionalPreference, Preferences
};
use project_tracker_core::{import_source_code_todos, Database, DatabaseMessage, Project, ProjectId, SerializableColor, SortMode, Task, TaskId, TaskTagId};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use iced::{
	alignment::{Alignment, Horizontal},
	keyboard,
	widget::{
		column, container, row,
		text, text_input, Row, Space,
	},
	Color, Element,
	Length::Fill,
	Padding, Subscription,
};
use iced_aw::{drop_down, DropDown};
use once_cell::sync::Lazy;
use std::{
	collections::HashSet,
	time::SystemTime,
};

static PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static SEARCH_TASKS_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum ProjectPageMessage {
	RefreshCachedTaskList,

	OpenSortModeDropdown,
	CloseSortModeDropdown,
	SetSortMode(SortMode),

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

	ConfirmDeleteProject,

	AnimateProgressbar,
}

impl From<ProjectPageMessage> for Message {
	fn from(value: ProjectPageMessage) -> Self {
		ContentPageMessage::ProjectPageMessage(value).into()
	}
}

#[derive(Debug, Clone)]
pub struct CachedTaskList {
	pub todo: Vec<TaskId>,
	pub done: Vec<TaskId>,
	pub source_code_todo: Vec<TaskId>,
	cache_time: SystemTime,
}

impl CachedTaskList {
	pub fn new(todo: Vec<TaskId>, done: Vec<TaskId>, source_code_todo: Vec<TaskId>) -> Self {
		Self {
			todo,
			done,
			source_code_todo,
			cache_time: SystemTime::now(),
		}
	}

	pub fn generate(
		project: &Project,
		task_tag_filter: &HashSet<TaskTagId>,
		search_filter: &Option<String>,
		sort_unspecified_tasks_at_bottom: bool
	) -> Self {
		let matches = |task: &Task| {
			task.matches_filter(task_tag_filter)
				&& search_filter
					.as_ref()
					.map(|search_filter| {
						SkimMatcherV2::default()
							.fuzzy_match(task.name(), search_filter)
							.or(SkimMatcherV2::default()
								.fuzzy_match(task.description(), search_filter))
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

		project.sort_mode.sort(project, &mut todo_list, sort_unspecified_tasks_at_bottom);
		project.sort_mode.sort(project, &mut done_list, sort_unspecified_tasks_at_bottom);
		project.sort_mode.sort(project, &mut source_code_todo_list, sort_unspecified_tasks_at_bottom);
		Self::new(todo_list, done_list, source_code_todo_list)
	}
}

#[derive(Debug, Clone)]
pub struct ProjectPage {
	pub project_id: ProjectId,
	pub cached_task_list: CachedTaskList,
	show_done_tasks: bool,
	show_source_code_todos: bool,
	show_color_picker: bool,
	pub filter_task_tags: HashSet<TaskTagId>,
	search_tasks_filter: Option<String>,
	progressbar_animation: ScalarAnimation,
	previous_project_progress: f32,
	show_context_menu: bool,
	show_sort_mode_dropdown: bool,
	pub importing_source_code_todos: bool,
}

impl ProjectPage {
	pub fn new(project_id: ProjectId, project: &Project, preferences: &Option<Preferences>) -> Self {
		let cached_task_list = CachedTaskList::generate(project, &HashSet::new(), &None, preferences.sort_unspecified_tasks_at_bottom());

		Self {
			project_id,
			cached_task_list,
			show_done_tasks: false,
			show_source_code_todos: true,
			show_color_picker: false,
			filter_task_tags: HashSet::new(),
			search_tasks_filter: None,
			progressbar_animation: ScalarAnimation::Idle,
			previous_project_progress: project.get_completion_percentage(),
			show_context_menu: false,
			show_sort_mode_dropdown: false,
			importing_source_code_todos: false,
		}
	}

	pub fn update(
		&mut self,
		message: ProjectPageMessage,
		database: &Option<Database>,
		preferences: &Option<Preferences>
	) -> ContentPageAction {
		let command = match message {
			ProjectPageMessage::RefreshCachedTaskList => ContentPageAction::None,

			ProjectPageMessage::OpenSortModeDropdown => { self.show_sort_mode_dropdown = true; ContentPageAction::None },
			ProjectPageMessage::CloseSortModeDropdown => { self.show_sort_mode_dropdown = false; ContentPageAction::None },
			ProjectPageMessage::SetSortMode(new_sort_mode) => {
				self.show_sort_mode_dropdown = false;
				DatabaseMessage::ChangeProjectSortMode {
					project_id: self.project_id,
					new_sort_mode
				}
				.into()
			}

			ProjectPageMessage::ShowContextMenu => { self.show_context_menu = true; ContentPageAction::None },
			ProjectPageMessage::HideContextMenu => { self.show_context_menu = false; ContentPageAction::None },
			ProjectPageMessage::OpenManageTaskTagsModal => { self.show_context_menu = false; ContentPageAction::OpenManageTaskTagsModal(self.project_id) },

			ProjectPageMessage::OpenSearchTasks => {
				self.search_tasks_filter = Some(String::new());
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				text_input::focus(SEARCH_TASKS_TEXT_INPUT_ID.clone()).into()
			}
			ProjectPageMessage::CloseSearchTasks => {
				self.search_tasks_filter = None;
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				ContentPageAction::None
			}
			ProjectPageMessage::ChangeSearchTasksFilter(new_filter) => {
				self.search_tasks_filter = Some(new_filter);
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				ContentPageAction::None
			}

			ProjectPageMessage::ImportSourceCodeTodosDialog => {
				self.show_context_menu = false;
				self.importing_source_code_todos = true;
				iced::Task::perform(Self::pick_todo_source_folders_dialog(), |folders| {
					if let Some(folders) = folders {
						ProjectPageMessage::ImportSourceCodeTodos(folders).into()
					} else {
						ProjectPageMessage::ImportSourceCodeTodosDialogCanceled.into()
					}
				})
				.into()
			}
			ProjectPageMessage::ImportSourceCodeTodosDialogCanceled => {
				self.importing_source_code_todos = false;
				ContentPageAction::None
			},
			ProjectPageMessage::ImportSourceCodeTodos(todos) => {
				self.importing_source_code_todos = false;
				DatabaseMessage::ImportSourceCodeTodos{
					project_id: self.project_id,
					source_code_todo_tasks: todos
				}
				.into()
			}

			ProjectPageMessage::ShowSourceCodeTodos(show) => {
				self.show_source_code_todos = show;
				ContentPageAction::None
			}

			ProjectPageMessage::ShowDoneTasks(show) => {
				self.show_done_tasks = show;
				ContentPageAction::None
			}

			ProjectPageMessage::ToggleFilterTaskTag(task_tag_id) => {
				if self.filter_task_tags.contains(&task_tag_id) {
					self.filter_task_tags.remove(&task_tag_id);
				} else {
					self.filter_task_tags.insert(task_tag_id);
				}
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				ContentPageAction::None
			}
			ProjectPageMessage::UnsetFilterTaskTag(task_tag_id) => {
				self.filter_task_tags.remove(&task_tag_id);
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				ContentPageAction::None
			}

			ProjectPageMessage::ShowColorPicker => {
				self.show_color_picker = true;
				ContentPageAction::None
			}
			ProjectPageMessage::HideColorPicker => {
				self.show_color_picker = false;
				ContentPageAction::None
			}
			ProjectPageMessage::ChangeProjectColor(new_color) => {
				self.show_color_picker = false;
				DatabaseMessage::ChangeProjectColor {
					project_id: self.project_id,
					new_color: SerializableColor::from_iced_color(new_color),
				}
				.into()
			}

			ProjectPageMessage::ConfirmDeleteProject => {
				self.show_context_menu = false;

				let project_name = database.as_ref().and_then(|db|
					db.get_project(&self.project_id).map(|project| project.name.clone())
				)
				.unwrap_or("<invalid project id>".to_string());

				ContentPageAction::ConfirmDeleteProject {
					project_id: self.project_id,
					project_name,
				}
			},

			ProjectPageMessage::AnimateProgressbar => {
				self.progressbar_animation.update();
				ContentPageAction::None
			}
		};

		if let Some(database_ref) = database {
			if self.cached_task_list.cache_time < *database_ref.last_changed_time() {
				self.generate_cached_task_list(database_ref, preferences);
			}
		}

		command
	}

	pub fn subscription(&self) -> Subscription<ProjectPageMessage> {
		Subscription::batch([
			self.progressbar_animation
				.subscription()
				.map(|_| ProjectPageMessage::AnimateProgressbar),
			keyboard::on_key_press(|key, modifiers| match key.as_ref() {
				keyboard::Key::Character("f") if modifiers.command() => {
					Some(ProjectPageMessage::OpenSearchTasks)
				},
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
						&app.task_ui_id_map,
						&app.task_description_markdown_items,
						app.dragged_task,
						app.just_minimal_dragging,
						app.sidebar_page.task_dropzone_hovered,
						self.show_done_tasks,
						self.show_source_code_todos,
						self.importing_source_code_todos
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
			text_input("New project name", &project.name)
				.id(PROJECT_NAME_TEXT_INPUT_ID.clone())
				.size(TITLE_TEXT_SIZE)
				.on_input(|edited_name| {
					DatabaseMessage::ChangeProjectName {
						project_id: self.project_id,
						new_name: edited_name.clone(),
					}
					.into()
				})
				.style(|t, s| text_input_style_borderless(t, s, false))
				.into();//text_input_style_default),

		let show_color_picker_button = edit_color_palette_button(
			project.color.to_iced_color(),
			self.show_color_picker,
			if self.show_color_picker {
				ProjectPageMessage::HideColorPicker.into()
			} else {
				ProjectPageMessage::ShowColorPicker.into()
			},
		);

		let color_picker = DropDown::new(
			show_color_picker_button,
			color_palette(project.color.to_iced_color(), |c| {
				ProjectPageMessage::ChangeProjectColor(c).into()
			}),
			self.show_color_picker
		)
		.width(Fill)
		.alignment(drop_down::Alignment::End)
		.on_dismiss(ProjectPageMessage::HideColorPicker.into());

		let task_tags_list: Vec<Element<Message>> = project.task_tags.iter()
			.map(|(tag_id, tag)| {
				task_tag_button(tag, self.filter_task_tags.contains(&tag_id))
					.on_press(ProjectPageMessage::ToggleFilterTaskTag(tag_id).into())
					.into()
			})
			.collect();

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
				quick_actions
			]
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center)]
			.width(Fill),
			spacer(),
			row![
				container("Tags:").padding(HORIZONTAL_SCROLLABLE_PADDING),
				horizontal_scrollable(Row::with_children(task_tags_list).spacing(SPACING_AMOUNT))
					.width(Fill),
				container(
					sort_dropdown_button(self.show_sort_mode_dropdown, project.sort_mode)
				)
				.padding(HORIZONTAL_SCROLLABLE_PADDING),
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
				container(open_create_task_modal_button())
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

	pub fn generate_cached_task_list(&mut self, database: &Database, preferences: &Option<Preferences>) {
		if let Some(project) = database.get_project(&self.project_id) {
			self.cached_task_list = CachedTaskList::generate(
				project,
				&self.filter_task_tags,
				&self.search_tasks_filter,
				preferences.sort_unspecified_tasks_at_bottom()
			);

			// update progress bar animation
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

	async fn pick_todo_source_folders_dialog() -> Option<Vec<Task>> {
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Import Todos from source files")
			.pick_folders()
			.await;

		file_dialog_result.map(|folder_handles| {
			let folder_paths = folder_handles.iter()
				.map(|folder_handle| folder_handle.path().to_path_buf())
				.collect();
			import_source_code_todos(folder_paths)
		})
	}
}
