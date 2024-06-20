use iced::{
    advanced::Application,
    font::{Family, Stretch, Style, Weight},
    window::{self, Level, Position},
    Font, Pixels, Settings, Size,
};
use pumpbin::Pumpbin;

fn main() -> iced::Result {
    let size = Size::new(1000.0, 600.0);

    let settings = Settings {
        id: Some(env!("CARGO_PKG_NAME").into()),
        window: window::Settings {
            size,
            position: Position::Centered,
            min_size: Some(size),
            visible: true,
            resizable: true,
            decorations: true,
            transparent: false,
            level: Level::Normal,
            exit_on_close_request: true,
            ..Default::default()
        },
        fonts: vec![include_bytes!("../assets/JetBrainsMonoNerdFontPropo-Regular.ttf").into()],
        default_font: Font {
            family: Family::Name("JetBrainsMono Nerd Font"),
            weight: Weight::Normal,
            stretch: Stretch::Normal,
            style: Style::Normal,
        },
        default_text_size: Pixels(13.0),
        antialiasing: true,
        ..Default::default()
    };

    Pumpbin::run(settings)
}
