use chrono::NaiveDate;
use iced_aw::date_picker::Date;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SerializableDate {
	pub year: i32,
	pub month: u32,
	pub day: u32,
}

impl PartialOrd for SerializableDate {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

// 2024.cmp(2025) -> Less
impl Ord for SerializableDate {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match self.year.cmp(&other.year) {
			Ordering::Equal => {
				match self.month.cmp(&other.month) {
					Ordering::Equal => self.day.cmp(&other.day),
					other => other,
				}
			},
			other => other
		}
	}
}


impl From<SerializableDate> for Date {
	fn from(value: SerializableDate) -> Self {
		Self::from_ymd(value.year, value.month, value.day)
	}
}

impl From<Date> for SerializableDate {
	fn from(value: Date) -> Self {
		Self {
			year: value.year,
			month: value.month,
			day: value.day,
		}
	}
}

impl From<NaiveDate> for SerializableDate {
	fn from(value: NaiveDate) -> Self {
		let date: Date = value.into();
		date.into()
	}
}