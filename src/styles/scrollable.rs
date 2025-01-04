use iced::{
	border::{rounded, Border},
	widget::{
		container,
		scrollable::{self, Rail, Status, Style},
	},
	Theme,
};

pub fn scrollable_style(theme: &Theme, status: Status) -> Style {
	let active_rail = Rail {
		background: None,
		border: Border::default(),
		scroller: scrollable::Scroller {
			color: theme.extended_palette().background.weak.color,
			border: rounded(f32::MAX),
		},
	};

	match status {
		Status::Active => Style {
			container: container::Style::default(),
			gap: None,
			horizontal_rail: active_rail,
			vertical_rail: active_rail,
		},
		Status::Hovered {
			is_horizontal_scrollbar_hovered,
			is_vertical_scrollbar_hovered,
		} => {
			let hovered_rail = Rail {
				background: None,
				border: Border::default(),
				scroller: scrollable::Scroller {
					color: theme.extended_palette().background.strong.color,
					border: rounded(f32::MAX),
				},
			};

			Style {
				container: container::Style::default(),
				gap: None,
				horizontal_rail: if is_horizontal_scrollbar_hovered {
					hovered_rail
				} else {
					active_rail
				},
				vertical_rail: if is_vertical_scrollbar_hovered {
					hovered_rail
				} else {
					active_rail
				},
			}
		}
		Status::Dragged {
			is_horizontal_scrollbar_dragged,
			is_vertical_scrollbar_dragged,
		} => {
			let rail = Rail {
				background: None,
				border: Border::default(),
				scroller: scrollable::Scroller {
					color: theme.extended_palette().primary.base.color,
					border: rounded(f32::MAX),
				},
			};

			Style {
				container: container::Style::default(),
				gap: None,
				horizontal_rail: if is_horizontal_scrollbar_dragged {
					rail
				} else {
					active_rail
				},
				vertical_rail: if is_vertical_scrollbar_dragged {
					rail
				} else {
					active_rail
				},
			}
		}
	}
}

pub fn logs_scrollable_style(theme: &Theme, status: Status) -> Style {
	let background = Some(theme.extended_palette().background.base.color.into());

	let active_rail = Rail {
		background,
		border: Border::default(),
		scroller: scrollable::Scroller {
			color: theme.extended_palette().background.strong.color,
			border: rounded(f32::MAX),
		},
	};

	match status {
		Status::Active => Style {
			container: container::Style::default(),
			gap: None,
			horizontal_rail: active_rail,
			vertical_rail: active_rail,
		},
		Status::Hovered {
			is_horizontal_scrollbar_hovered,
			is_vertical_scrollbar_hovered,
		} => {
			let hovered_rail = Rail {
				background,
				border: Border::default(),
				scroller: scrollable::Scroller {
					color: theme.extended_palette().background.strong.color,
					border: rounded(f32::MAX),
				},
			};

			Style {
				container: container::Style::default(),
				gap: None,
				horizontal_rail: if is_horizontal_scrollbar_hovered {
					hovered_rail
				} else {
					active_rail
				},
				vertical_rail: if is_vertical_scrollbar_hovered {
					hovered_rail
				} else {
					active_rail
				},
			}
		}
		Status::Dragged {
			is_horizontal_scrollbar_dragged,
			is_vertical_scrollbar_dragged,
		} => {
			let rail = Rail {
				background,
				border: Border::default(),
				scroller: scrollable::Scroller {
					color: theme.extended_palette().primary.base.color,
					border: rounded(f32::MAX),
				},
			};

			Style {
				container: container::Style::default(),
				gap: None,
				horizontal_rail: if is_horizontal_scrollbar_dragged {
					rail
				} else {
					active_rail
				},
				vertical_rail: if is_vertical_scrollbar_dragged {
					rail
				} else {
					active_rail
				},
			}
		}
	}
}
