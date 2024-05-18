use iced::widget::{progress_bar, ProgressBar};
use crate::styles::CompletionBarStyle;

pub fn completion_bar(completion: f32) -> ProgressBar {
	progress_bar(0.0..=100.0, completion * 100.0)
		.style(iced::theme::ProgressBar::Custom(Box::new(CompletionBarStyle)))
		.height(5.0)
}