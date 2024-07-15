use std::iter;

use anyhow::anyhow;
use iced::{
    advanced::graphics::image::image_rs::ImageFormat,
    window::{self, Level, Position},
    Font, Pixels, Settings, Size, Task,
};
use memchr::memmem;
use rand::RngCore;
use rfd::{AsyncMessageDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

pub const JETBRAINS_MONO_FONT: Font = Font::with_name("JetBrainsMono NF");

pub fn error_dialog(error: anyhow::Error) {
    MessageDialog::new()
        .set_buttons(MessageButtons::Ok)
        .set_description(error.to_string())
        .set_level(MessageLevel::Error)
        .set_title("PumpBin")
        .show();
}

pub fn message_dialog(message: String, level: MessageLevel) -> Task<MessageDialogResult> {
    let dialog = AsyncMessageDialog::new()
        .set_buttons(MessageButtons::Ok)
        .set_description(message)
        .set_level(level)
        .set_title("PumpBin")
        .show();
    Task::future(dialog)
}

pub fn settings() -> Settings {
    Settings {
        fonts: vec![include_bytes!("../assets/JetBrainsMonoNerdFont-Regular.ttf").into()],
        default_font: JETBRAINS_MONO_FONT,
        default_text_size: Pixels(13.0),
        antialiasing: true,
        ..Default::default()
    }
}

pub fn window_settings() -> window::Settings {
    let size = Size::new(1000.0, 600.0);

    window::Settings {
        size,
        position: Position::Centered,
        min_size: Some(size),
        visible: true,
        resizable: true,
        decorations: true,
        transparent: false,
        level: Level::Normal,
        icon: window::icon::from_file_data(
            include_bytes!("../logo/icon.png"),
            Some(ImageFormat::Png),
        )
        .ok(),
        exit_on_close_request: true,
        ..Default::default()
    }
}

pub fn replace(
    bin: &mut [u8],
    holder: &[u8],
    replace_by: &[u8],
    max_len: usize,
) -> anyhow::Result<()> {
    let mut replace_by = replace_by.to_owned();

    let position = memmem::find_iter(bin, holder)
        .next()
        .ok_or(anyhow!("Not found {}", String::from_utf8_lossy(holder)))?;
    let mut random: Vec<u8> = iter::repeat(b'0')
        .take(max_len - replace_by.len())
        .collect();
    rand::thread_rng().fill_bytes(&mut random);
    replace_by.extend_from_slice(random.as_slice());

    bin[position..(position + max_len)].copy_from_slice(replace_by.as_slice());

    Ok(())
}
