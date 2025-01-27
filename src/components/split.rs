use crate::project_tracker::Message;
use iced::{
	widget::{
		pane_grid,
		pane_grid::{Configuration, ResizeEvent},
	},
	Border, Color, Element,
};

enum SplitPane {
	Sidebar,
	ContentPage,
}

pub struct Split {
	state: pane_grid::State<SplitPane>,
}

impl Split {
	pub fn new(ratio: f32) -> Self {
		Self {
			state: Self::state_config(ratio),
		}
	}

	pub fn resize(&mut self, ratio: f32) {
		self.state = Self::state_config(ratio);
	}

	pub fn view<'a>(
		&'a self,
		sidebar: impl Fn() -> Element<'a, Message>,
		content_page: impl Fn() -> Element<'a, Message>,
		on_resize: impl Fn(ResizeEvent) -> Message + 'a,
	) -> Element<'a, Message> {
		pane_grid(&self.state, |_pane, split_pane, _is_maximized| {
			pane_grid::Content::new(match split_pane {
				SplitPane::Sidebar => sidebar(),
				SplitPane::ContentPage => content_page(),
			})
		})
		.on_resize(5.0, on_resize)
		.style(|theme| pane_grid::Style {
			hovered_region: pane_grid::Highlight {
				background: Color {
					a: 0.5,
					..theme.extended_palette().primary.base.color
				}
				.into(),
				border: Border {
					width: 2.0,
					color: theme.extended_palette().primary.strong.color,
					radius: 0.0.into(),
				},
			},
			hovered_split: pane_grid::Line {
				color: theme.extended_palette().primary.base.color,
				width: 2.0,
			},
			picked_split: pane_grid::Line {
				color: theme.extended_palette().primary.strong.color,
				width: 3.0,
			},
		})
		.into()
	}

	fn state_config(ratio: f32) -> pane_grid::State<SplitPane> {
		pane_grid::State::with_configuration(Configuration::Split {
			axis: pane_grid::Axis::Vertical,
			ratio,
			a: Box::new(Configuration::Pane(SplitPane::Sidebar)),
			b: Box::new(Configuration::Pane(SplitPane::ContentPage)),
		})
	}
}
