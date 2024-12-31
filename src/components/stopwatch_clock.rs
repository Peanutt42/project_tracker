use crate::{pages::format_stopwatch_duration, project_tracker::Message, styles::FIRA_SANS_FONT};
use iced::{
	alignment::{Horizontal, Vertical},
	widget::canvas::{path::Arc, stroke, Cache, Geometry, LineCap, Path, Program, Stroke, Text},
	Color, Radians, Renderer, Theme, Vector,
};
use std::f32::consts::PI;

#[derive(Debug)]
pub struct StopwatchClock {
	percentage: f32,
	label: String,
	sub_label: Option<String>,
	cache: Cache,
}

impl StopwatchClock {
	pub fn new(percentage: f32, seconds_left: f32, needed_seconds: Option<f32>) -> Self {
		Self {
			percentage,
			label: format_stopwatch_duration(seconds_left.round_ties_even() as i64),
			sub_label: needed_seconds.map(|needed_seconds| {
				format_stopwatch_duration(needed_seconds.round_ties_even() as i64)
			}),
			cache: Cache::new(),
		}
	}

	pub fn label(&self) -> &String {
		&self.label
	}

	pub fn set_percentage(&mut self, percentage: f32) {
		self.percentage = percentage;
		self.cache.clear();
	}

	pub fn set_seconds_left(&mut self, seconds_left: f32) {
		self.label = format_stopwatch_duration(seconds_left.round_ties_even() as i64);
		self.cache.clear();
	}

	pub fn set_needed_seconds(&mut self, needed_seconds: f32) {
		self.sub_label = Some(format_stopwatch_duration(
			needed_seconds.round_ties_even() as i64
		));
		self.cache.clear();
	}
}

impl Program<Message> for StopwatchClock {
	type State = ();

	fn draw(
		&self,
		_state: &Self::State,
		renderer: &Renderer,
		theme: &Theme,
		bounds: iced::Rectangle,
		_cursor: iced::advanced::mouse::Cursor,
	) -> Vec<Geometry> {
		let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
			let center = frame.center();
			const STROKE_WIDTH: f32 = 7.5;
			let radius = frame.width().min(frame.height()) / 2.0 - STROKE_WIDTH;

			let stroke = |color: Color| Stroke {
				width: STROKE_WIDTH,
				style: stroke::Style::Solid(color),
				line_cap: LineCap::Round,
				..Default::default()
			};

			let background = Path::circle(center, radius);
			frame.stroke(
				&background,
				stroke(theme.extended_palette().secondary.base.color),
			);

			if self.percentage < 1.0 {
				let left_arc = Path::new(|builder| {
					builder.arc(Arc {
						center,
						radius,
						start_angle: Radians(-0.5 * PI),
						end_angle: Radians(1.5 * PI - self.percentage.min(1.0) * 2.0 * PI),
					})
				});
				frame.stroke(
					&left_arc,
					stroke(theme.extended_palette().primary.base.color),
				);
			} else {
				let overflow_percentage = self.percentage - 1.0;
				let overflow_arc = Path::new(|builder| {
					builder.arc(Arc {
						center,
						radius,
						start_angle: Radians(-0.5 * PI),
						end_angle: Radians(-0.5 * PI - overflow_percentage.min(1.0) * 2.0 * PI),
					})
				});
				frame.stroke(
					&overflow_arc,
					stroke(theme.extended_palette().danger.base.color),
				);
			}

			let label_text_size = 60.0;
			let label_y_offset = label_text_size / 10.0;

			frame.fill_text(Text {
				content: self.label.clone(),
				position: center + Vector::new(0.0, label_y_offset),
				color: if self.percentage < 1.0 {
					theme.extended_palette().background.base.text
				} else {
					theme.extended_palette().danger.base.color
				},
				horizontal_alignment: Horizontal::Center,
				vertical_alignment: Vertical::Center,
				size: label_text_size.into(),
				font: FIRA_SANS_FONT,
				..Default::default()
			});

			if let Some(sub_label) = &self.sub_label {
				frame.fill_text(Text {
					content: sub_label.clone(),
					position: center + Vector::new(0.0, label_text_size + label_y_offset),
					color: theme.extended_palette().background.base.text,
					horizontal_alignment: Horizontal::Center,
					vertical_alignment: Vertical::Center,
					size: 25.0.into(),
					font: FIRA_SANS_FONT,
					..Default::default()
				});
			}
		});

		vec![geometry]
	}
}
