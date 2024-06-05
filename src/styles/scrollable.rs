use iced::{widget::{container, scrollable::{self, Appearance, Direction, Properties, StyleSheet}}, Border, Theme};
use crate::styles::LIGHT_DARK_GREEN;

pub fn scrollable_vertical_direction() -> Direction {
	Direction::Vertical(
		Properties::new()
			.scroller_width(6.0)
	)
}

pub struct ScrollableStyle;

impl StyleSheet for ScrollableStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
        Appearance {
            container: container::Appearance::default(),
            gap: None,
            scrollbar: scrollable::Scrollbar {
                background: None,
                border: Border::default(),
                scroller: scrollable::Scroller {
                    color: style.extended_palette().background.weak.color,
                    border: Border::with_radius(f32::MAX),
                },
            },
        }
	}

	fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> Appearance {
		if is_mouse_over_scrollbar {
            scrollable::Appearance {
            	container: container::Appearance::default(),
            	gap: None,
                scrollbar: scrollable::Scrollbar {
                    background: None,
                    border: Border::default(),
                    scroller: scrollable::Scroller {
                        color: style.extended_palette().background.strong.color,
                        border: Border::with_radius(f32::MAX),
                    },
                },
            }
        } else {
            self.active(style)
        }
	}

	fn dragging(&self, _style: &Self::Style) -> Appearance {
		scrollable::Appearance {
			container: container::Appearance::default(),
           	gap: None,
	        scrollbar: scrollable::Scrollbar {
	            background: None,
	            border: Border::default(),
	            scroller: scrollable::Scroller {
	                color: LIGHT_DARK_GREEN,
	                border: Border::with_radius(f32::MAX),
	            },
	        },
	    }
	}
}
