use iced_aw::date_picker::Date;
use chrono::NaiveDate;
use project_tracker_core::SerializableDate;

pub trait SerializableDateConversion {
	fn to_iced_date(&self) -> Date;
	fn from_iced_date(date: Date) -> Self;
	fn from_naive_date(date: NaiveDate) -> Self;
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
	fn from_naive_date(date: NaiveDate) -> Self {
		let date: Date = date.into();
		Self::from_iced_date(date)
	}
}