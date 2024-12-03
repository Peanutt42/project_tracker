use project_tracker_core::{Project, SerializableColor, SortMode, TaskId};
use crate::icons::Bootstrap;
use iced::Color;
use std::cmp::Ordering;

pub trait SortModeUI {
	const ALL: &'static [SortMode];
	fn as_str(&self) -> &'static str;
	fn icon(&self) -> Bootstrap;
	fn sort(&self, project: &Project, tasks: &mut [TaskId], sort_specific_tasks_at_bottom: bool);
}

impl SortModeUI for SortMode {
	const ALL: &'static [SortMode] = &[
		SortMode::Manual,
		SortMode::DueDate,
		SortMode::NeededTime,
	];

	fn as_str(&self) -> &'static str {
		match self {
			Self::Manual => "Manual",
			Self::DueDate => "Due Date",
			Self::NeededTime => "Needed Time",
		}
	}

	fn icon(&self) -> Bootstrap {
		match self {
			Self::Manual => Bootstrap::SortDown,
			Self::DueDate | Self::NeededTime => Bootstrap::SortNumericDown,
		}
	}

	fn sort(&self, project: &Project, tasks: &mut [TaskId], sort_unspecified_tasks_at_bottom: bool) {
		match self {
			Self::Manual => {},
			Self::DueDate => {
				tasks.sort_unstable_by(|task_id_a, task_id_b| {
					if let (Some(task_a), Some(task_b)) = (project.get_task(task_id_a), project.get_task(task_id_b)) {
						match (&task_a.due_date, &task_b.due_date) {
							(Some(due_date_a), Some(due_date_b)) => due_date_a.cmp(due_date_b),
							(Some(_due_date_a), None) => if sort_unspecified_tasks_at_bottom {
								Ordering::Less
							}
							else {
								Ordering::Greater
							},
							(None, Some(_due_date_b)) => if sort_unspecified_tasks_at_bottom {
								Ordering::Greater
							}
							else {
								Ordering::Less
							},
							(None, None) => Ordering::Equal,
						}
					}
					else {
						Ordering::Equal
					}
				});
			},
			Self::NeededTime => {
				tasks.sort_unstable_by(|task_id_a, task_id_b| {
					if let (Some(task_a), Some(task_b)) = (project.get_task(task_id_a), project.get_task(task_id_b)) {
						match (&task_a.needed_time_minutes, &task_b.needed_time_minutes) {
							(Some(needed_time_minutes_a), Some(needed_time_minutes_b)) => needed_time_minutes_a.cmp(needed_time_minutes_b),
							(Some(_due_date_a), None) => if sort_unspecified_tasks_at_bottom {
								Ordering::Less
							}
							else {
								Ordering::Greater
							},
							(None, Some(_due_date_b)) => if sort_unspecified_tasks_at_bottom {
								Ordering::Greater
							}
							else {
								Ordering::Less
							},
							(None, None) => Ordering::Equal,
						}
					}
					else {
						Ordering::Equal
					}
				});
			},
		}
	}
}


pub trait IcedColorConversion {
	fn to_iced_color(&self) -> Color;
	fn from_iced_color(color: Color) -> Self;
}

impl IcedColorConversion for SerializableColor {
	fn to_iced_color(&self) -> Color {
		Color::from_rgb8(self.0[0], self.0[1], self.0[2])
	}
	fn from_iced_color(color: Color) -> Self {
		Self([
			(color.r * 255.0) as u8,
			(color.g * 255.0) as u8,
			(color.b * 255.0) as u8,
		])
	}
}