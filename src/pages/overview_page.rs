use crate::{
	components::{
		calendar_navigation_button, calendar_today_button, calendar_view_button,
		horizontal_seperator, open_project_button, task_widget, vertical_seperator,
	},
	core::IcedColorConversion,
	pages,
	preferences::{FirstWeekday, SerializedOverviewPage},
	project_tracker,
	styles::{PADDING_AMOUNT, SPACING_AMOUNT},
	OptionalPreference, Preferences, ProjectTrackerApp,
};
use chrono::{DateTime, Datelike, Days, Duration, Local, NaiveDate, Utc, Weekday};
use iced::{
	alignment::{Horizontal, Vertical},
	widget::{column, container, row, text, Column, Row, Space},
	Element,
	Length::Fill,
};
use iced_aw::style::colors::RED;
use project_tracker_core::{Database, ProjectId, SerializableDate, TaskId, TaskType};
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeMap, HashMap},
	time::SystemTime,
};
use tracing::error;

#[derive(Debug, Clone)]
pub struct Page {
	tasks: BTreeMap<SerializableDate, HashMap<ProjectId, Vec<TaskId>>>,
	cache_time: SystemTime,
}

#[derive(Debug, Clone)]
pub enum Message {
	RefreshCachedTaskList,
	GoForward,
	GoBackward,
	GoToToday,
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		pages::Message::OverviewPage(value).into()
	}
}

impl Page {
	pub fn new(database: Option<&Database>) -> Self {
		let mut tasks: BTreeMap<SerializableDate, HashMap<ProjectId, Vec<TaskId>>> =
			BTreeMap::new();

		if let Some(database) = database {
			for (project_id, project) in database.projects().iter() {
				for (task_id, task, task_type) in project.iter() {
					if let Some(due_date) = &task.due_date {
						if matches!(task_type, TaskType::Todo | TaskType::SourceCodeTodo) {
							tasks
								.entry(*due_date)
								.or_default()
								.entry(project_id)
								.or_default()
								.push(task_id);
						}
					}
				}
			}
		}

		Self {
			tasks,
			cache_time: SystemTime::now(),
		}
	}

	pub fn update(
		&mut self,
		message: Message,
		database: Option<&Database>,
		preferences: &mut Option<Preferences>,
	) {
		match message {
			Message::RefreshCachedTaskList => {
				if let Some(database_ref) = database {
					let cache_date_time: DateTime<Utc> = self.cache_time.into();
					if cache_date_time < *database_ref.last_changed_time() {
						*self = Self::new(database);
					}
				}
			}
			Message::GoForward => {
				if let Some(preferences) = preferences {
					if let SerializedOverviewPage::Calendar { view } =
						*preferences.serialized_overview_page()
					{
						let first_weekday = preferences.first_day_of_week();
						let view = view.go_forward(first_weekday);
						preferences.set_serialized_overview_page(
							SerializedOverviewPage::Calendar { view },
						);
					}
				}
			}
			Message::GoBackward => {
				if let Some(preferences) = preferences {
					if let SerializedOverviewPage::Calendar { view } =
						*preferences.serialized_overview_page()
					{
						let first_weekday = preferences.first_day_of_week();
						let view = view.go_backward(first_weekday);
						preferences.set_serialized_overview_page(
							SerializedOverviewPage::Calendar { view },
						);
					}
				}
			}
			Message::GoToToday => {
				if let Some(preferences) = preferences {
					if let SerializedOverviewPage::Calendar { view } =
						*preferences.serialized_overview_page()
					{
						let view = match view {
							CalendarView::Week { .. } => CalendarView::current_week(),
							CalendarView::ThreeDays { .. } => CalendarView::current_three_days(),
						};
						preferences.set_serialized_overview_page(
							SerializedOverviewPage::Calendar { view },
						);
					}
				}
			}
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, project_tracker::Message> {
		let today = Local::now().date_naive();
		let first_week_day = app.preferences.first_day_of_week();
		let calendar_view = match app.preferences.serialized_overview_page() {
			SerializedOverviewPage::Calendar { view } => *view,
			_ => CalendarView::default(),
		};
		let week_days = calendar_view.days(first_week_day);

		let current_first_date = match calendar_view {
			CalendarView::Week { week_day } => week_day,
			CalendarView::ThreeDays { first_date } => first_date,
		};

		column![
			row![
				row![
					calendar_navigation_button(false),
					calendar_today_button(),
					calendar_navigation_button(true)
				],
				Space::new(SPACING_AMOUNT, 0.0),
				text(Self::view_range_label(&week_days)),
				Space::new(Fill, 0.0),
				calendar_view_button(
					CalendarView::Week {
						week_day: current_first_date
					},
					matches!(calendar_view, CalendarView::Week { .. }),
					true,
					false
				),
				calendar_view_button(
					CalendarView::ThreeDays {
						first_date: current_first_date
					},
					matches!(calendar_view, CalendarView::ThreeDays { .. }),
					false,
					true
				)
			]
			.align_y(Vertical::Center),
			Row::with_children(
				week_days
					.into_iter()
					.enumerate()
					.map(|(i, (week_day, day))| {
						Row::new()
							.push_maybe(if i == 0 {
								Some(vertical_seperator())
							} else {
								None
							})
							.push(Self::day_view(
								week_day,
								day,
								day == today,
								self.tasks.get(&day.into()),
								app,
							))
							.push(vertical_seperator())
							.into()
					})
			)
			.width(Fill)
		]
		.spacing(SPACING_AMOUNT)
		.padding(PADDING_AMOUNT)
		.width(Fill)
		.height(Fill)
		.into()
	}

	fn day_view<'a>(
		week_day: Weekday,
		day: NaiveDate,
		today: bool,
		tasks: Option<&'a HashMap<ProjectId, Vec<TaskId>>>,
		app: &'a ProjectTrackerApp,
	) -> Element<'a, project_tracker::Message> {
		let tasks: Element<project_tracker::Message> = match tasks {
			Some(tasks) => Column::with_children(tasks.iter().map(
				|(project_id, task_ids)| -> Element<project_tracker::Message> {
					let task_widgets = task_ids.iter().map(|task_id| {
						match app
							.database
							.ok()
							.and_then(|db| db.get_project_task_type(project_id, task_id))
						{
							Some((project, task, task_type)) => task_widget(
								task,
								*task_id,
								app.task_ui_id_map.get_dropzone_id_mut(*task_id),
								task_type,
								*project_id,
								project,
								app.preferences.code_editor(),
								false,
								true,
								false,
								false,
								false,
								true,
							),
							None => text("<invalid project or task id>").into(),
						}
					});
					match app.database.ok().and_then(|db| db.get_project(project_id)) {
						Some(project) => column![
							open_project_button(
								*project_id,
								&project.name,
								project.color.to_iced_color()
							),
							Column::with_children(task_widgets).spacing(SPACING_AMOUNT)
						]
						.width(Fill)
						.padding(PADDING_AMOUNT)
						.spacing(SPACING_AMOUNT)
						.into(),
						None => text("<invalid project or task id>").into(),
					}
				},
			))
			.width(Fill)
			.spacing(SPACING_AMOUNT)
			.into(),
			None => Space::new(0, 0).into(),
		};

		container(
			column![
				text!("{week_day:?}"),
				text!("{}", day.day0() + 1).color_maybe(if today { Some(RED) } else { None }),
				horizontal_seperator(),
				tasks,
			]
			.align_x(Horizontal::Center),
		)
		.center_x(Fill)
		.into()
	}

	fn view_range_label(week_days: &[(Weekday, NaiveDate)]) -> String {
		let month_str = |month0: u32| -> String {
			match month0 {
				0 => "January".to_string(),
				1 => "Febuary".to_string(),
				2 => "March".to_string(),
				3 => "April".to_string(),
				4 => "May".to_string(),
				5 => "June".to_string(),
				6 => "July".to_string(),
				7 => "August".to_string(),
				8 => "September".to_string(),
				9 => "October".to_string(),
				10 => "November".to_string(),
				11 => "December".to_string(),
				_ => {
					error!("invalid month0 index (0..=11): {month0}");
					format!("<invalid month0 index: {month0}>")
				}
			}
		};

		match week_days.first() {
			Some((_, first_date)) => {
				let first_month0 = first_date.month0();
				let (_, first_year) = first_date.year_ce();

				match week_days.last() {
					Some((_, last_date)) if first_month0 != last_date.month0() => {
						let (_, last_year) = last_date.year_ce();
						format!(
							"{}{} - {} {last_year}",
							month_str(first_month0),
							if first_year == last_year {
								String::new()
							} else {
								format!(" {first_year}")
							},
							month_str(last_date.month0())
						)
					}
					_ => format!("{} {first_year}", month_str(first_month0)),
				}
			}
			None => {
				error!("empty week days!");
				"<empty week days>".to_string()
			}
		}
	}
}

/// `ThreeDays`: 3 days, initially pervious, current and next day
/// `Week`: Mon to Sun / Sun to Sat
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CalendarView {
	ThreeDays { first_date: NaiveDate },
	Week { week_day: NaiveDate },
}

impl Default for CalendarView {
	fn default() -> Self {
		Self::current_week()
	}
}

impl CalendarView {
	pub fn current_week() -> Self {
		Self::Week {
			week_day: Local::now().date_naive(),
		}
	}

	pub fn current_three_days() -> Self {
		let today = Local::now().date_naive();
		Self::ThreeDays {
			first_date: today.pred_opt().unwrap_or(today),
		}
	}

	pub fn label(&self) -> &'static str {
		match self {
			Self::Week { .. } => "Week",
			Self::ThreeDays { .. } => "3 Days",
		}
	}

	pub fn go_forward(self, first_week_day: FirstWeekday) -> Self {
		match self {
			Self::Week { week_day } => Self::Week {
				week_day: week_day
					.week(first_week_day.as_week_day())
					.first_day()
					.checked_add_days(Days::new(7))
					.unwrap_or(week_day),
			},
			Self::ThreeDays { first_date } => Self::ThreeDays {
				first_date: first_date.succ_opt().unwrap_or(first_date),
			},
		}
	}

	pub fn go_backward(self, first_week_day: FirstWeekday) -> Self {
		match self {
			Self::Week { week_day } => Self::Week {
				week_day: week_day
					.week(first_week_day.as_week_day())
					.first_day()
					.checked_sub_days(Days::new(7))
					.unwrap_or(week_day),
			},
			Self::ThreeDays { first_date } => Self::ThreeDays {
				first_date: first_date.pred_opt().unwrap_or(first_date),
			},
		}
	}

	pub fn days(&self, first_week_day: FirstWeekday) -> Vec<(Weekday, NaiveDate)> {
		let (first_date, num_days) = match self {
			Self::Week { week_day } => {
				let week = week_day.week(first_week_day.as_week_day());
				(week.first_day(), 7)
			}
			Self::ThreeDays { first_date } => (*first_date, 3),
		};
		(0..num_days)
			.map(|i| {
				let date = first_date + Duration::days(i);
				let week_day = date.weekday();
				(week_day, date)
			})
			.collect()
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use chrono::{NaiveDate, Weekday};

	use crate::{pages::overview_page::CalendarView, preferences::FirstWeekday};

	#[test]
	fn test_week_days() {
		let today_date = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
		let first_week_day = FirstWeekday::Monday;

		assert_eq!(
			CalendarView::Week {
				week_day: today_date
			}
			.days(first_week_day),
			[
				(Weekday::Mon, NaiveDate::from_ymd_opt(2025, 1, 27).unwrap()),
				(Weekday::Tue, NaiveDate::from_ymd_opt(2025, 1, 28).unwrap()),
				(Weekday::Wed, NaiveDate::from_ymd_opt(2025, 1, 29).unwrap()),
				(Weekday::Thu, NaiveDate::from_ymd_opt(2025, 1, 30).unwrap()),
				(Weekday::Fri, NaiveDate::from_ymd_opt(2025, 1, 31).unwrap()),
				(Weekday::Sat, NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()),
				(Weekday::Sun, NaiveDate::from_ymd_opt(2025, 2, 2).unwrap())
			]
		);
	}

	#[test]
	fn test_three_days() {
		let today_date = NaiveDate::from_ymd_opt(2025, 1, 30).unwrap();
		let first_week_day = FirstWeekday::Monday;

		assert_eq!(
			CalendarView::ThreeDays {
				first_date: today_date
			}
			.days(first_week_day),
			[
				(Weekday::Thu, NaiveDate::from_ymd_opt(2025, 1, 30).unwrap()),
				(Weekday::Fri, NaiveDate::from_ymd_opt(2025, 1, 31).unwrap()),
				(Weekday::Sat, NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()),
			]
		);
	}
}
