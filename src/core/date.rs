use chrono::{Datelike, Local, Timelike};
use iced_aw::date_picker::Date;
use project_tracker_core::SerializableDate;

use crate::DateFormatting;

pub trait SerializableDateConversion {
	fn to_iced_date(&self) -> Date;
	fn from_iced_date(date: Date) -> Self;
}

impl SerializableDateConversion for SerializableDate {
	fn to_iced_date(&self) -> Date {
		Date::from_ymd(self.year, self.month, self.day)
	}
	fn from_iced_date(date: Date) -> Self {
		Self {
			year: date.year,
			month: date.month,
			day: date.day,
		}
	}
}

pub fn formatted_date_time(date_formatting: DateFormatting) -> String {
	let now = Local::now();
	match date_formatting {
		DateFormatting::DayMonthYear => format!(
			"{}_{}_{} - {}_{}",
			now.day(),
			now.month(),
			now.year(),
			now.hour(),
			now.minute()
		),
		DateFormatting::MonthDayYear => format!(
			"{}_{}_{} - {}_{}",
			now.month(),
			now.day(),
			now.year(),
			now.hour(),
			now.minute()
		),
	}
}
