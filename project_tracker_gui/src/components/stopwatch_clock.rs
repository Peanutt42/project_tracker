use std::f32::consts::PI;

use iced::{alignment::{Horizontal, Vertical}, widget::canvas::{path::Arc, stroke, Cache, Geometry, LineCap, Path, Program, Stroke, Text}, Color, Font, Point, Radians, Renderer, Theme};

use crate::project_tracker::UiMessage;

#[derive(Debug)]
pub struct StopwatchClock {
	percentage: f32,
	label: String,
	sub_label: String,
	cache: Cache,
}

impl StopwatchClock {
	pub fn new(percentage: f32, label: String, sub_label: String) -> Self {
		Self {
			percentage,
			label,
			sub_label,
			cache: Cache::new()
		}
	}

	pub fn label(&self) -> &String { &self.label }

	pub fn set_percentage(&mut self, percentage: f32) {
		self.percentage = percentage;
		self.cache.clear();
	}

	pub fn set_label(&mut self, label: String) {
		self.label = label;
		self.cache.clear();
	}

	pub fn set_sub_label(&mut self, sub_label: String) {
		self.sub_label = sub_label;
		self.cache.clear();
	}
}

impl Program<UiMessage> for StopwatchClock {
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

			let stroke = |color: Color| {
				Stroke {
					width: STROKE_WIDTH,
					style: stroke::Style::Solid(color),
					line_cap: LineCap::Round,
					..Default::default()
				}
			};

			let background = Path::circle(center, radius);
			frame.stroke(&background, stroke(theme.extended_palette().secondary.base.color));

			if self.percentage < 1.0 {
				let left_arc = Path::new(|builder| builder.arc(Arc {
					center,
					radius,
					start_angle: Radians(-0.5 * PI),
					end_angle: Radians(1.5 * PI - self.percentage.min(1.0) * 2.0 * PI)
				}));
				frame.stroke(
					&left_arc,
					stroke(theme.extended_palette().primary.base.color)
				);
			}
			else {
				let overflow_percentage = self.percentage - 1.0;
				let overflow_arc = Path::new(|builder| builder.arc(Arc {
					center,
					radius,
					start_angle: Radians(-0.5 * PI),
					end_angle: Radians(-0.5 * PI - overflow_percentage.min(1.0) * 2.0 * PI)
				}));
				frame.stroke(&overflow_arc, stroke(theme.extended_palette().danger.base.color));
			}

			frame.fill_text(Text {
				content: self.label.clone(),
				position: center,
				color: if self.percentage < 1.0 {
					theme.extended_palette().background.base.text
				}
				else {
					theme.extended_palette().danger.base.color
				},
				horizontal_alignment: Horizontal::Center,
				vertical_alignment: Vertical::Center,
				size: 60.0.into(),
				font: Font::DEFAULT,
				..Default::default()
			});

			frame.fill_text(Text {
				content: self.sub_label.clone(),
				position: Point { x: center.x, y: center.y + 60.0 },
				color: theme.extended_palette().background.base.text,
				horizontal_alignment: Horizontal::Center,
				vertical_alignment: Vertical::Center,
				size: 25.0.into(),
				font: Font::DEFAULT,
				..Default::default()
			});
		});

		vec![geometry]
	}
}