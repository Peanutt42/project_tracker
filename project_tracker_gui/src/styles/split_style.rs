use iced::Theme;
use iced_aw::split::{Appearance, StyleSheet};

pub struct SplitStyle;

impl StyleSheet for SplitStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();
		Appearance {
			border_width: 0.0,
			divider_border_width: 1.0,
			divider_border_color: palette.background.base.color,
			divider_background: palette.background.weak.color.into(),
			..Default::default()
		}
	}

	fn dragged(&self, style: &Self::Style) -> Appearance {
        let palette = style.extended_palette();
		Appearance {
			border_width: 0.0,
			divider_border_width: 100.0,
			divider_border_color: palette.background.strong.color,
			divider_background: palette.background.strong.color.into(),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
        let palette = style.extended_palette();
		Appearance {
			border_width: 0.0,
			divider_border_width: 1.0,
			divider_border_color: palette.background.base.color,
			divider_background: palette.background.strong.color.into(),
			..Default::default()
		}
	}
}