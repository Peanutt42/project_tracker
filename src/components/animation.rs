use iced::{window, Subscription};
use std::time::Instant;

#[derive(Debug, Default, Clone)]
pub enum ScalarAnimation {
	#[default]
	Idle,
	Animating {
		start: f32,
		target: f32,
		value: f32,
		start_time: Instant,
		duration_secs: f32,
	},
}

impl ScalarAnimation {
	pub fn start(start: f32, target: f32, duration_secs: f32) -> Self {
		Self::Animating {
			start,
			target,
			value: start,
			start_time: Instant::now(),
			duration_secs,
		}
	}

	pub fn subscription(&self) -> Subscription<()> {
		if matches!(self, ScalarAnimation::Animating { .. }) {
			window::frames().map(|_at| ())
		} else {
			Subscription::none()
		}
	}

	pub fn update(&mut self) {
		if let ScalarAnimation::Animating {
			start,
			target,
			value,
			start_time,
			duration_secs,
			..
		} = self
		{
			let anim_percentage =
				Instant::now().duration_since(*start_time).as_secs_f32() / *duration_secs;
			if anim_percentage > 1.0 {
				*self = ScalarAnimation::Idle;
			} else {
				*value = *start + (*target - *start) * anim_percentage;
			}
		}
	}

	pub fn get_value(&self) -> Option<f32> {
		match self {
			ScalarAnimation::Idle => None,
			ScalarAnimation::Animating { value, .. } => Some(*value),
		}
	}
}
