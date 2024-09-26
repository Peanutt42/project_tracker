use iced::widget::{progress_bar, ProgressBar};
use crate::styles::completion_bar_style;

pub fn completion_bar(completion: f32) -> ProgressBar<'static> {
	progress_bar(0.0..=100.0, completion * 100.0)
		.style(completion_bar_style)
		.height(5.0)
}