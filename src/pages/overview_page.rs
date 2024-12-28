use std::{collections::{BTreeMap, HashMap}, time::SystemTime};
use chrono::{DateTime, Days, NaiveDate, Utc};
use iced::{widget::{column, row, container, container::Id, text, Column}, Element, Length::Fill, Padding};
use iced_aw::date_picker::Date;
use crate::{components::{open_project_button, overview_time_section_button, task_widget, vertical_scrollable}, core::{IcedColorConversion, SerializableDateConversion, TASK_TAG_QUAD_HEIGHT}, pages::ContentPageMessage, project_tracker::Message, styles::{PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TINY_SPACING_AMOUNT}, OptionalPreference, Preferences, ProjectTrackerApp};
use crate::core::SortModeUI;
use project_tracker_core::{Database, ProjectId, SerializableDate, SortMode, Task, TaskId};

#[derive(Debug, Clone)]
pub struct OverviewPage {
	overdue_tasks: BTreeMap<SerializableDate, HashMap<ProjectId, Vec<TaskId>>>, // sorted by due date, then by project
	today_tasks: HashMap<ProjectId, Vec<TaskId>>, // sorted by est. needed time
	tomorrow_tasks: HashMap<ProjectId, Vec<TaskId>>, // sorted by est. needed time
	future_tasks: BTreeMap<SerializableDate, HashMap<ProjectId, Vec<TaskId>>>, // sorted by due date, then by project
	show_overdue_tasks: bool,
	show_today_tasks: bool,
	show_tomorrow_tasks: bool,
	show_future_tasks: bool,
	cache_time: SystemTime,
}

#[derive(Debug, Clone)]
pub enum OverviewPageMessage {
	RefreshCachedTaskList,
	ToggleShowOverdueTasks,
	ToggleShowTodayTasks,
	ToggleShowTomorrowTasks,
	ToggleShowFutureTasks,
}

impl From<OverviewPageMessage> for Message {
	fn from(value: OverviewPageMessage) -> Self {
		ContentPageMessage::OverviewPageMessage(value).into()
	}
}

impl OverviewPage {
	pub fn new(database: Option<&Database>, preferences: &Option<Preferences>) -> Self {
		let mut overdue_tasks: BTreeMap<SerializableDate, HashMap<ProjectId, Vec<TaskId>>> = BTreeMap::new();
		let mut today_tasks: HashMap<ProjectId, Vec<TaskId>> = HashMap::new();
		let mut tomorrow_tasks: HashMap<ProjectId, Vec<TaskId>> = HashMap::new();
		let mut future_tasks: BTreeMap<SerializableDate, HashMap<ProjectId, Vec<TaskId>>> = BTreeMap::new();

		if let Some(database) = database {
			let today: NaiveDate = Date::today().into();
			let today_date: Date = today.into();

			let tomorrow = today.checked_add_days(Days::new(1));
			let tomorrow_date: Option<Date> = tomorrow.map(|date| date.into());

			for (project_id, project) in database.projects().iter() {
				let mut cache_overdue_tasks = |task_id: TaskId, task: &Task| {
					if let Some(due_date) = &task.due_date {
						if *due_date < SerializableDate::from_iced_date(today_date) {
							overdue_tasks.entry(*due_date)
								.or_default()
								.entry(project_id)
								.or_default()
								.push(task_id);
						}
					}
				};
				let mut cache_today_tasks = |task_id: TaskId, task: &Task| {
					if let Some(due_date) = &task.due_date {
						if *due_date == SerializableDate::from_iced_date(today_date) {
							today_tasks.entry(project_id)
								.or_default()
								.push(task_id);
						}
					}
				};
				let mut cache_tomorrow_tasks = |task_id: TaskId, task: &Task| {
					if let Some(tomorrow_date) = tomorrow_date {
						if let Some(due_date) = &task.due_date {
							if *due_date == SerializableDate::from_iced_date(tomorrow_date) {
								tomorrow_tasks.entry(project_id)
									.or_default()
									.push(task_id);
							}
						}
					}
				};
				let mut cache_future_tasks = |task_id: TaskId, task: &Task| {
					if let Some(tomorrow_date) = tomorrow_date {
						if let Some(due_date) = &task.due_date {
							if *due_date > SerializableDate::from_iced_date(tomorrow_date) {
								future_tasks.entry(*due_date)
									.or_default()
									.entry(project_id)
									.or_default()
									.push(task_id);
							}
						}
					}
				};

				for (task_id, task) in project.todo_tasks.iter() {
					cache_overdue_tasks(task_id, task);
					cache_today_tasks(task_id, task);
					cache_tomorrow_tasks(task_id, task);
					cache_future_tasks(task_id, task);
				}
				for (task_id, task) in project.source_code_todos.iter() {
					cache_overdue_tasks(*task_id, task);
					cache_today_tasks(*task_id, task);
					cache_tomorrow_tasks(*task_id, task);
					cache_future_tasks(*task_id, task);
				}
			}

			let sort_unspecified_tasks_at_bottom = preferences.sort_unspecified_tasks_at_bottom();
			for (project_id, tasks) in today_tasks.iter_mut() {
				if let Some(project) = database.get_project(project_id) {
					SortMode::NeededTime.sort(project, tasks, sort_unspecified_tasks_at_bottom);
				}
			}
			for (project_id, tasks) in tomorrow_tasks.iter_mut() {
				if let Some(project) = database.get_project(project_id) {
					SortMode::NeededTime.sort(project, tasks, sort_unspecified_tasks_at_bottom);
				}
			}
		}

		Self {
			overdue_tasks,
			today_tasks,
			tomorrow_tasks,
			future_tasks,
			show_overdue_tasks: true,
			show_today_tasks: true,
			show_tomorrow_tasks: true,
			show_future_tasks: true,
			cache_time: SystemTime::now(),
		}
	}

	pub fn update(&mut self, message: OverviewPageMessage, database: Option<&Database>, preferences: &Option<Preferences>) {
		match message {
			OverviewPageMessage::RefreshCachedTaskList => {
				if let Some(database_ref) = database {
					let cache_date_time: DateTime<Utc> = self.cache_time.into();
					if cache_date_time < *database_ref.last_changed_time() {
						*self = Self::new(database, preferences);
					}
				}
			},
			OverviewPageMessage::ToggleShowOverdueTasks => self.show_overdue_tasks = !self.show_overdue_tasks,
			OverviewPageMessage::ToggleShowTodayTasks => self.show_today_tasks = !self.show_today_tasks,
			OverviewPageMessage::ToggleShowTomorrowTasks => self.show_tomorrow_tasks = !self.show_tomorrow_tasks,
			OverviewPageMessage::ToggleShowFutureTasks => self.show_future_tasks = !self.show_future_tasks,
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, Message> {
		let overdue_tasks_len: usize = self.overdue_tasks.values()
			.map(|tasks|
				tasks.values()
					.map(|tasks| tasks.len())
					.sum::<usize>()
			)
			.sum();

		let today_tasks_len: usize = self.today_tasks.values()
			.map(|tasks| tasks.len())
			.sum();

		let tomorrow_tasks_len: usize = self.tomorrow_tasks.values()
			.map(|tasks| tasks.len())
			.sum();

		let future_tasks_len: usize = self.future_tasks.values()
			.map(|tasks|
				tasks.values()
					.map(|tasks| tasks.len())
					.sum::<usize>()
			)
			.sum();

		container(
			vertical_scrollable(
				column![
					Self::view_tasks_for_days(
						"Overdue",
						&self.overdue_tasks,
						!self.show_overdue_tasks,
						OverviewPageMessage::ToggleShowOverdueTasks.into(),
						overdue_tasks_len,
						app
					),

					Self::view_tasks_for_day(
						"Today",
						today_tasks_len,
						!self.show_today_tasks,
						OverviewPageMessage::ToggleShowTodayTasks.into(),
						&self.today_tasks,
						app
					),

					Self::view_tasks_for_day(
						"Tomorrow",
						tomorrow_tasks_len,
						!self.show_tomorrow_tasks,
						OverviewPageMessage::ToggleShowTomorrowTasks.into(),
						&self.tomorrow_tasks,
						app
					),

					Self::view_tasks_for_days(
						"Future",
						&self.future_tasks,
						!self.show_future_tasks,
						OverviewPageMessage::ToggleShowFutureTasks.into(),
						future_tasks_len,
						app
					),
				]
				.width(Fill)
				.spacing(SPACING_AMOUNT)
				.padding(PADDING_AMOUNT)
			)
		)
		.width(Fill)
		.height(Fill)
		.into()
	}

	fn view_tasks_for_days<'a>(
		label: &'static str,
		tasks: &'a BTreeMap<SerializableDate, HashMap<ProjectId, Vec<TaskId>>>,
		collapsed: bool,
		on_toggle_collabsed: Message,
		tasks_len: usize,
		app: &'a ProjectTrackerApp
	) -> Element<'a, Message>
	{
		Column::new()
			.push(overview_time_section_button(
				label,
				tasks_len,
				collapsed,
				on_toggle_collabsed
			))
			.push_maybe(if tasks.is_empty() || collapsed {
				None
			} else {
				Some(
					Column::with_children(tasks.iter()
						.map(|(_date, tasks)| {
							Self::view_tasks(tasks, app, true)
						}))
						.spacing(SPACING_AMOUNT)
				)
			})
			.spacing(SPACING_AMOUNT)
			.into()
	}

	fn view_tasks_for_day<'a>(
		time_label: &'static str,
		task_count: usize,
		collapsed: bool,
		on_toggle_collabsed: Message,
		tasks: &'a HashMap<ProjectId, Vec<TaskId>>,
		app: &'a ProjectTrackerApp
	) -> Element<'a, Message>
	{
		Column::new()
			.push(overview_time_section_button(time_label, task_count, collapsed, on_toggle_collabsed))
			.push_maybe(if tasks.is_empty() || collapsed {
				None
			} else {
				Some(Self::view_tasks(tasks, app, false))
			})
			.spacing(SPACING_AMOUNT)
			.into()
	}

	fn view_tasks<'a>(tasks: &'a HashMap<ProjectId, Vec<TaskId>>, app: &'a ProjectTrackerApp, show_due_date: bool) -> Element<'a, Message> {
		Column::with_children(
			tasks.iter()
				.map(|(project_id, tasks)| {
					if let Some(project) = app.database.as_ref().and_then(|db| db.get_project(project_id)) {
						let list = Column::with_children(
							tasks.iter()
								.map(|task_id| {
									if let Some((task, task_type)) = project.get_task_and_type(task_id) {
										task_widget(
											task,
											*task_id,
											app.task_ui_id_map.get_dropzone_id(*task_id).unwrap_or(Id::unique()),
											task_type,
											app.task_description_markdown_items.get(task_id),
											*project_id,
											project,
											false,
											true,
											false,
											false,
											show_due_date
										)
									}
									else {
										text("<invalid task id>").into()
									}
								})
						)
						.spacing(SMALL_SPACING_AMOUNT);

						let first_task_has_tags = tasks.first()
							.and_then(|task_id|
								project.get_task(task_id)
									.map(|task| !task.tags.is_empty())
							)
							.unwrap_or(false);

						row![
							container(
								open_project_button(*project_id, &project.name, project.color.to_iced_color())
							)
							.width(120.0)
							.padding(Padding::default().top(
								if first_task_has_tags {
									TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT
								} else {
									0.0
								}
							)),

							list.padding(Padding::default().left(PADDING_AMOUNT)),
						]
						.into()
					}
					else {
						Element::new(text("<invalid project id>"))
					}
				})
		)
		.spacing(SPACING_AMOUNT)
		.padding(Padding::default().left(PADDING_AMOUNT))
		.into()
	}
}