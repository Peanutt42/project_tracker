use iced::{widget::{container, scrollable::{self, Appearance, StyleSheet}}, Border, Theme};

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

	fn dragging(&self, style: &Self::Style) -> Appearance {
		scrollable::Appearance {
			container: container::Appearance::default(),
			gap: None,
			scrollbar: scrollable::Scrollbar {
				background: None,
				border: Border::default(),
				scroller: scrollable::Scroller {
					color: style.extended_palette().success.base.color,
					border: Border::with_radius(f32::MAX),
				},
			},
		}
	}
}
