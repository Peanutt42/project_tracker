use crate::styles::{
	background_shadow_color, color_average, mix_color, text_color, BLUR_RADIUS, BORDER_RADIUS,
	GREY, LARGE_BLUR_RADIUS, LARGE_BORDER_RADIUS,
};
use iced::{
	border::{rounded, Radius},
	widget::button::{Status, Style},
	Border, Color, Shadow, Theme,
};

pub fn project_preview_style(
	theme: &Theme,
	status: Status,
	selected: bool,
	project_color: Option<Color>,
) -> Style {
	let border = rounded(BORDER_RADIUS);

	match status {
		Status::Active => {
			let background_color = if selected {
				project_color.unwrap_or(theme.extended_palette().primary.base.color)
			} else {
				mix_color(
					theme.extended_palette().background.weak.color,
					theme.extended_palette().background.base.color,
					0.75,
				)
			};

			Style {
				background: Some(background_color.into()),
				text_color: text_color(background_color),
				border,
				..Default::default()
			}
		}
		Status::Hovered | Status::Pressed | Status::Disabled => {
			let background_color = if selected {
				project_color.unwrap_or(theme.extended_palette().primary.weak.color)
			} else {
				color_average(
					theme.extended_palette().background.weak.color,
					theme.extended_palette().background.base.color,
				)
			};

			Style {
				background: Some(background_color.into()),
				text_color: text_color(background_color),
				border,
				..Default::default()
			}
		}
	}
}

pub fn overview_button_style(theme: &Theme, status: Status, selected: bool) -> Style {
	let border = rounded(BORDER_RADIUS);

	match status {
		Status::Active => {
			let background_color = if selected {
				theme.extended_palette().primary.base.color
			} else {
				mix_color(
					theme.extended_palette().background.weak.color,
					theme.extended_palette().background.base.color,
					0.75,
				)
			};

			Style {
				background: Some(background_color.into()),
				text_color: text_color(background_color),
				border,
				..Default::default()
			}
		}
		Status::Hovered | Status::Pressed | Status::Disabled => {
			let background_color = if selected {
				theme.extended_palette().primary.weak.color
			} else {
				color_average(
					theme.extended_palette().background.weak.color,
					theme.extended_palette().background.base.color,
				)
			};

			Style {
				background: Some(background_color.into()),
				text_color: text_color(background_color),
				border,
				..Default::default()
			}
		}
	}
}

pub fn stopwatch_page_button_style(
	theme: &Theme,
	status: Status,
	selected: bool,
	timer_ticking: bool,
	dropzone_highlight: bool,
) -> Style {
	let border = rounded(BORDER_RADIUS);

	let stopwatch_color = if timer_ticking || dropzone_highlight {
		theme.extended_palette().danger.base.color
	} else {
		theme.extended_palette().primary.base.color
	};

	match status {
		Status::Active => {
			let background_color = if selected || timer_ticking || dropzone_highlight {
				stopwatch_color
			} else {
				mix_color(
					theme.extended_palette().background.weak.color,
					theme.extended_palette().background.base.color,
					0.75,
				)
			};

			Style {
				background: Some(background_color.into()),
				text_color: text_color(background_color),
				border,
				..Default::default()
			}
		}
		Status::Hovered | Status::Pressed | Status::Disabled => {
			let background_color = if selected || timer_ticking || dropzone_highlight {
				stopwatch_color
			} else {
				color_average(
					theme.extended_palette().background.weak.color,
					theme.extended_palette().background.base.color,
				)
			};

			Style {
				background: Some(background_color.into()),
				text_color: text_color(background_color),
				border,
				..Default::default()
			}
		}
	}
}

pub fn hidden_secondary_button_style(theme: &Theme, status: Status) -> Style {
	match status {
		Status::Active => Style {
			background: None,
			text_color: theme.palette().text,
			border: rounded(BORDER_RADIUS),
			..Style::default()
		},
		Status::Hovered | Status::Pressed => Style {
			background: Some(
				color_average(
					theme.extended_palette().background.weak.color,
					theme.extended_palette().background.base.color,
				)
				.into(),
			),
			..hidden_secondary_button_style(theme, Status::Active)
		},
		Status::Disabled => Style {
			text_color: GREY,
			..hidden_secondary_button_style(theme, Status::Active)
		},
	}
}

pub fn dangerous_button_style(theme: &Theme, status: Status) -> Style {
	match status {
		Status::Active => Style {
			background: Some(theme.extended_palette().danger.base.color.into()),
			text_color: theme.extended_palette().danger.base.text,
			border: rounded(BORDER_RADIUS),
			..Style::default()
		},
		Status::Hovered => Style {
			background: Some(Color::from_rgb(0.8, 0.0, 0.0).into()),
			..dangerous_button_style(theme, Status::Active)
		},
		Status::Pressed | Status::Disabled => Style {
			background: Some(Color::from_rgb(0.6, 0.0, 0.0).into()),
			..dangerous_button_style(theme, Status::Active)
		},
	}
}

pub fn delete_done_tasks_button_style(theme: &Theme, status: Status) -> Style {
	match status {
		Status::Active => Style {
			background: Some(theme.extended_palette().secondary.base.color.into()),
			text_color: theme.extended_palette().secondary.base.text,
			border: rounded(BORDER_RADIUS),
			..Style::default()
		},
		Status::Hovered => Style {
			background: Some(Color::from_rgb(0.8, 0.0, 0.0).into()),
			text_color: theme.extended_palette().danger.base.text,
			..delete_done_tasks_button_style(theme, Status::Active)
		},
		Status::Pressed => Style {
			background: Some(Color::from_rgb(0.6, 0.0, 0.0).into()),
			..delete_done_tasks_button_style(theme, Status::Hovered)
		},
		Status::Disabled => Style::default(),
	}
}

pub fn circle_button_style(theme: &Theme, status: Status) -> Style {
	let pair = theme.extended_palette().primary.base;
	match status {
		Status::Active => Style {
			background: Some(pair.color.into()),
			text_color: pair.text,
			border: rounded(f32::MAX),
			..Default::default()
		},
		Status::Hovered | Status::Pressed => Style {
			background: Some(
				mix_color(
					pair.color,
					theme.extended_palette().background.strong.color,
					0.25,
				)
				.into(),
			),
			text_color: pair.text,
			..circle_button_style(theme, Status::Active)
		},
		Status::Disabled => Style {
			background: Some(
				color_average(pair.color, theme.extended_palette().background.strong.color).into(),
			),
			text_color: pair.text,
			..circle_button_style(theme, Status::Active)
		},
	}
}

pub fn primary_button_style(theme: &Theme, status: Status) -> Style {
	let pair = theme.extended_palette().primary.base;
	match status {
		Status::Active => Style {
			background: Some(pair.color.into()),
			text_color: pair.text,
			border: rounded(BORDER_RADIUS),
			..Style::default()
		},
		Status::Hovered | Status::Pressed => Style {
			background: Some(
				mix_color(
					pair.color,
					theme.extended_palette().background.strong.color,
					0.25,
				)
				.into(),
			),
			text_color: pair.text,
			..primary_button_style(theme, Status::Active)
		},
		Status::Disabled => Style {
			background: Some(
				color_average(pair.color, theme.extended_palette().background.strong.color).into(),
			),
			text_color: pair.text,
			..primary_button_style(theme, Status::Active)
		},
	}
}

pub fn secondary_button_style_default(theme: &Theme, status: Status) -> Style {
	secondary_button_style(theme, status, true, true, true, true)
}

pub fn secondary_button_style_no_rounding(theme: &Theme, status: Status) -> Style {
	secondary_button_style(theme, status, false, false, false, false)
}

pub fn secondary_button_style_only_round_left(theme: &Theme, status: Status) -> Style {
	secondary_button_style(theme, status, true, true, false, false)
}

pub fn secondary_button_style_only_round_right(theme: &Theme, status: Status) -> Style {
	secondary_button_style(theme, status, false, false, true, true)
}

pub fn secondary_button_style_only_round_top(theme: &Theme, status: Status) -> Style {
	secondary_button_style(theme, status, true, false, true, false)
}

pub fn secondary_button_style(
	theme: &Theme,
	status: Status,
	round_left_top: bool,
	round_left_bottom: bool,
	round_right_top: bool,
	round_right_bottom: bool,
) -> Style {
	let border_radius = Radius::default()
		.top_left(if round_left_top { BORDER_RADIUS } else { 0.0 })
		.top_right(if round_right_top { BORDER_RADIUS } else { 0.0 })
		.bottom_right(if round_right_bottom {
			BORDER_RADIUS
		} else {
			0.0
		})
		.bottom_left(if round_left_bottom {
			BORDER_RADIUS
		} else {
			0.0
		});

	match status {
		Status::Active => Style {
			background: Some(theme.extended_palette().secondary.base.color.into()),
			text_color: theme.extended_palette().secondary.base.text,
			border: rounded(border_radius),
			..Style::default()
		},
		Status::Hovered | Status::Pressed => Style {
			background: Some(theme.extended_palette().background.strong.color.into()),
			text_color: theme.extended_palette().secondary.base.text,
			..secondary_button_style(
				theme,
				Status::Active,
				round_left_top,
				round_left_bottom,
				round_right_top,
				round_right_bottom,
			)
		},
		Status::Disabled => Style::default(),
	}
}

pub fn delete_button_style(
	theme: &Theme,
	status: Status,
	round_top_left: bool,
	round_top_right: bool,
	round_bottom_left: bool,
	round_bottom_right: bool,
) -> Style {
	let active_style = Style {
		background: Some(theme.extended_palette().secondary.base.color.into()),
		text_color: theme.extended_palette().secondary.base.text,
		border: rounded(
			Radius::default()
				.top_left(if round_top_left { BORDER_RADIUS } else { 0.0 })
				.top_right(if round_top_right { BORDER_RADIUS } else { 0.0 })
				.bottom_left(if round_bottom_left {
					BORDER_RADIUS
				} else {
					0.0
				})
				.bottom_right(if round_bottom_right {
					BORDER_RADIUS
				} else {
					0.0
				}),
		),
		..Default::default()
	};

	match status {
		Status::Active => active_style,
		Status::Hovered => Style {
			background: Some(theme.extended_palette().danger.base.color.into()),
			..active_style
		},
		Status::Pressed => Style {
			background: Some(
				color_average(
					theme.extended_palette().background.base.color,
					theme.extended_palette().danger.weak.color,
				)
				.into(),
			),
			..active_style
		},
		Status::Disabled => active_style, // No disabled state defined, fallback to active
	}
}

pub fn selection_list_button_style(
	theme: &Theme,
	status: Status,
	selected: bool,
	round_left_top: bool,
	round_right_top: bool,
	round_left_bottom: bool,
	round_right_bottom: bool,
) -> Style {
	let active_style = Style {
		background: Some(if selected {
			theme.extended_palette().primary.strong.color.into()
		} else {
			theme.extended_palette().secondary.base.color.into()
		}),
		border: rounded(
			Radius::default()
				.top_left(if round_left_top { BORDER_RADIUS } else { 0.0 })
				.top_right(if round_right_top { BORDER_RADIUS } else { 0.0 })
				.bottom_left(if round_left_bottom {
					BORDER_RADIUS
				} else {
					0.0
				})
				.bottom_right(if round_right_bottom {
					BORDER_RADIUS
				} else {
					0.0
				}),
		),
		text_color: if selected {
			theme.extended_palette().primary.base.text
		} else {
			theme.extended_palette().secondary.base.text
		},
		..Default::default()
	};

	match status {
		Status::Active => active_style,
		Status::Hovered | Status::Pressed => Style {
			background: Some(if selected {
				theme.extended_palette().primary.strong.color.into()
			} else {
				theme.extended_palette().background.strong.color.into()
			}),
			..active_style
		},
		Status::Disabled => active_style,
	}
}

pub fn enum_dropdown_button_style(
	theme: &Theme,
	status: Status,
	selected: bool,
	round_top: bool,
	round_bottom: bool,
) -> Style {
	let active_style = Style {
		background: Some(if selected {
			theme.extended_palette().primary.strong.color.into()
		} else {
			theme.extended_palette().secondary.base.color.into()
		}),
		border: rounded(
			Radius::default()
				.top(if round_top { BORDER_RADIUS } else { 0.0 })
				.bottom(if round_bottom { BORDER_RADIUS } else { 0.0 }),
		),
		text_color: if selected {
			theme.extended_palette().primary.base.text
		} else {
			theme.extended_palette().secondary.base.text
		},
		..Default::default()
	};

	match status {
		Status::Active => active_style,
		Status::Hovered | Status::Pressed => Style {
			background: Some(if selected {
				theme.extended_palette().primary.strong.color.into()
			} else {
				theme.extended_palette().background.strong.color.into()
			}),
			..active_style
		},
		Status::Disabled => active_style,
	}
}

pub fn task_tag_button_style(theme: &Theme, status: Status, color: Color, toggled: bool) -> Style {
	let active_style = Style {
		background: Some(if toggled {
			color.into()
		} else {
			theme.extended_palette().background.base.color.into()
		}),
		text_color: if toggled {
			text_color(color)
		} else {
			theme.extended_palette().background.base.text
		},
		border: Border {
			color,
			width: 1.0,
			radius: LARGE_BORDER_RADIUS.into(),
		},
		..Default::default()
	};

	match status {
		Status::Active => active_style,
		Status::Hovered => Style {
			background: Some(if toggled {
				color.into()
			} else {
				color_average(color, theme.extended_palette().background.base.color).into()
			}),
			..active_style
		},
		Status::Pressed => Style {
			background: Some(color.into()),
			..active_style
		},
		Status::Disabled => active_style,
	}
}

pub fn task_button_style(theme: &Theme, status: Status, dragging: bool) -> Style {
	let active_style = Style {
		background: None,
		text_color: theme.palette().text,
		border: rounded(BORDER_RADIUS),
		..Default::default()
	};

	match status {
		Status::Active => active_style,
		Status::Hovered | Status::Pressed => Style {
			background: Some(
				color_average(
					theme.extended_palette().background.weak.color,
					theme.extended_palette().background.base.color,
				)
				.into(),
			),
			shadow: if dragging {
				Shadow::default()
			} else {
				Shadow {
					color: background_shadow_color(theme.extended_palette()),
					blur_radius: BLUR_RADIUS,
					..Default::default()
				}
			},
			..active_style
		},
		Status::Disabled => active_style,
	}
}

pub fn settings_tab_button_style(theme: &Theme, status: Status, selected: bool) -> Style {
	let active_style = Style {
		background: if selected {
			Some(theme.extended_palette().primary.strong.color.into())
		} else {
			None
		},
		text_color: if selected {
			theme.extended_palette().primary.strong.text
		} else {
			theme.extended_palette().background.base.text
		},
		border: rounded(BORDER_RADIUS),
		..Default::default()
	};

	match status {
		Status::Active => active_style,
		Status::Hovered | Status::Pressed => Style {
			background: if selected {
				Some(theme.extended_palette().primary.strong.color.into())
			} else {
				Some(theme.extended_palette().secondary.base.color.into())
			},
			..active_style
		},
		Status::Disabled => active_style,
	}
}

pub fn timer_button_style(theme: &Theme, status: Status, timer_ticking: bool) -> Style {
	let pair = if timer_ticking {
		theme.extended_palette().danger.base
	} else {
		theme.extended_palette().primary.base
	};

	let active_style = Style {
		background: Some(pair.color.into()),
		text_color: pair.text,
		border: rounded(15.0),
		..Default::default()
	};

	match status {
		Status::Active => active_style,
		Status::Hovered | Status::Pressed => {
			let color = mix_color(
				pair.color,
				theme.extended_palette().background.strong.color,
				0.25,
			);
			Style {
				background: Some(color.into()),
				text_color: pair.text,
				shadow: Shadow {
					color: Color { a: 0.2, ..color },
					blur_radius: LARGE_BLUR_RADIUS,
					..Default::default()
				},
				..active_style
			}
		}
		Status::Disabled => active_style,
	}
}
