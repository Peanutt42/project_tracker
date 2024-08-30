mod button_styles;
pub use button_styles::{ProjectPreviewButtonStyle, HiddenSecondaryButtonStyle, DangerousButtonStyle, DeleteDoneTasksButtonStyle, DeleteButtonStyle, SecondaryButtonStyle, SelectionListButtonStyle, PaletteItemButtonStyle, ColorPaletteButtonStyle, InvisibleButtonStyle, TaskTagButtonStyle, TaskButtonStyle};

mod container_styles;
pub use container_styles::{RoundedContainerStyle, PaletteContainerStyle, ProjectPreviewBackgroundContainerStyle, TaskBackgroundContainerStyle, DropzoneContainerStyle, InBetweenDropzoneContainerStyle, TooltipContainerStyle, ShadowContainerStyle};

mod completion_bar_style;
pub use completion_bar_style::CompletionBarStyle;

mod split_style;
pub use split_style::SplitStyle;

mod checkbox_style;
pub use checkbox_style::GreenCheckboxStyle;

mod text_input;
pub use text_input::TextInputStyle;

mod text_editor;
pub use text_editor::TextEditorStyle;

mod scrollable;
pub use scrollable::{ScrollableStyle, scrollable_horizontal_direction, scrollable_vertical_direction, SCROLLBAR_WIDTH};

mod modal_style;
pub use modal_style::{ModalStyle, PaletteModalStyle, ModalCardStyle};

mod constants;
pub use constants::{
	colors::{color_average, mix_color, background_shadow_color, background_shadow_alpha, GREY, SELECTION_COLOR},
	padding::{SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, PADDING_AMOUNT, LARGE_PADDING_AMOUNT},
	size::{SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, HEADING_TEXT_SIZE, TITLE_TEXT_SIZE, MINIMAL_DRAG_DISTANCE},
	spacing::{TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, LARGE_SPACING_AMOUNT},
	border_radius::{BORDER_RADIUS, LARGE_BORDER_RADIUS},
	blur_radius::{SMALL_BLUR_RADIUS, BLUR_RADIUS},
	GAP,
};

mod text;
pub use text::{strikethrough_text, text_color, primary_text_color, disabled_primary_text_color};

mod theme;
pub use theme::ProjectTrackerTheme;
