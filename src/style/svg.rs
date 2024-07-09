use iced::{widget::svg, Theme};

pub fn svg_primary_base(theme: &Theme, _: svg::Status) -> svg::Style {
    svg::Style {
        color: Some(theme.extended_palette().primary.base.color),
    }
}
