use iced::{
    widget::button::{Status, Style},
    Theme,
};

pub fn selected(theme: &Theme, _: Status) -> Style {
    let palette = theme.extended_palette();
    let mut style = Style::default();
    style.border.width = 1.0;
    style.text_color = palette.success.base.color;
    style.border.color = palette.success.base.color;
    style
}

pub fn unselected(theme: &Theme, _: Status) -> Style {
    let palette = theme.extended_palette();
    let mut style = Style::default();
    style.border.width = 1.0;
    style.text_color = palette.primary.base.color;
    style.border.color = palette.primary.base.color;
    style
}
