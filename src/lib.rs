mod button_style;
pub mod plugin;
pub mod svg_style;

use std::{fmt::Display, fs, iter, ops::Not, path::PathBuf, time::Duration};

use dirs::{desktop_dir, home_dir};
use iced::{
    advanced::{graphics::image::image_rs::ImageFormat, Application},
    executor,
    widget::{
        button, column, container, horizontal_rule, pick_list, row, scrollable,
        svg::{self, Handle},
        text, text_editor, text_input, vertical_rule, Scrollable, Svg,
    },
    window::{self, Level, Position},
    Alignment, Background, Element, Font, Length, Pixels, Renderer, Settings, Size, Task, Theme,
};
use memchr::memmem;
use plugin::{EncryptType, Plugin, Plugins};
use rand::RngCore;
use rfd::AsyncFileDialog;

pub const FONT: Font = Font::with_name("JetBrainsMono NF");

#[derive(Debug)]
pub struct Pumpbin {
    shellcode_src: String,
    shellcode_save_type: ShellcodeSaveType,
    supported_binary_types: Vec<BinaryType>,
    selected_binary_type: Option<BinaryType>,
    message: String,
    supported_platforms: Vec<Platform>,
    selected_platform: Option<Platform>,
    plugins: Plugins,
    selected_plugin: Option<String>,
    encrypt_type: EncryptType,
    plugin_desc: text_editor::Content,
    selected_theme: Theme,
}

impl Default for Pumpbin {
    fn default() -> Self {
        Self {
            shellcode_src: Default::default(),
            shellcode_save_type: ShellcodeSaveType::Local,
            supported_binary_types: Default::default(),
            selected_binary_type: None,
            message: "Welcome to PumpBin!".into(),
            supported_platforms: Default::default(),
            selected_platform: None,
            plugins: if let Ok(plugins) = Plugins::reade_plugins() {
                plugins
            } else {
                Plugins::default()
            },
            selected_plugin: None,
            encrypt_type: EncryptType::None,
            plugin_desc: text_editor::Content::with_text("None"),
            selected_theme: Theme::CatppuccinMacchiato,
        }
    }
}

impl Pumpbin {
    pub fn settings() -> Settings {
        let size = Size::new(1000.0, 600.0);

        Settings {
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
                icon: match window::icon::from_file_data(
                    include_bytes!("../logo/icon.png"),
                    Some(ImageFormat::Png),
                ) {
                    Ok(x) => Some(x),
                    Err(_) => None,
                },
                exit_on_close_request: true,
                ..Default::default()
            },
            fonts: vec![include_bytes!("../assets/JetBrainsMonoNerdFont-Regular.ttf").into()],
            default_font: FONT,
            default_text_size: Pixels(13.0),
            antialiasing: true,
            ..Default::default()
        }
    }

    fn random_encrypt_pass(&mut self) {
        match self.encrypt_type() {
            EncryptType::None => self.encrypt_type = EncryptType::None,
            EncryptType::Xor(x) => {
                let mut pass = x.clone();
                rand::thread_rng().fill_bytes(&mut pass);
                self.encrypt_type = EncryptType::Xor(pass);
            }
            EncryptType::AesGcm(x) => {
                let mut pass = x.clone();
                rand::thread_rng().fill_bytes(pass.key_holder_mut());
                rand::thread_rng().fill_bytes(pass.nonce_holder_mut());
                self.encrypt_type = EncryptType::AesGcm(pass);
            }
        }
    }

    fn show_message(&mut self, message: String) -> Task<Message> {
        self.message = message;
        let wait = async {
            tokio::time::sleep(Duration::from_secs(3)).await;
        };
        Task::perform(wait, Message::ClearMessage)
    }

    pub fn shellcode_src(&self) -> &str {
        &self.shellcode_src
    }

    pub fn shellcode_save_type(&self) -> ShellcodeSaveType {
        self.shellcode_save_type
    }

    pub fn supported_binary_types(&self) -> &[BinaryType] {
        &self.supported_binary_types
    }

    pub fn selected_binary_type(&self) -> Option<BinaryType> {
        self.selected_binary_type
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn supported_platforms(&self) -> &[Platform] {
        &self.supported_platforms
    }

    pub fn selected_platform(&self) -> Option<Platform> {
        self.selected_platform
    }

    pub fn plugins(&self) -> &Plugins {
        &self.plugins
    }

    pub fn selected_plugin(&self) -> Option<&String> {
        self.selected_plugin.as_ref()
    }

    pub fn encrypt_type(&self) -> &EncryptType {
        &self.encrypt_type
    }

    pub fn plugin_desc(&self) -> &text_editor::Content {
        &self.plugin_desc
    }

    pub fn selected_theme(&self) -> Theme {
        self.selected_theme.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ShellcodeSrcChanged(String),
    ChooseShellcodeClicked,
    ChooseShellcodeDone(Option<PathBuf>),
    EncryptShellcode(Option<PathBuf>),
    EncryptShellcodeDone(Result<(), String>),
    PlatformChanged(Platform),
    GenerateClicked,
    GenerateDone(Result<(), String>),
    BinaryTypeChanged(BinaryType),
    AddPluginClicked,
    AddPluginDone(Result<(u32, u32, Plugins), String>),
    RemovePlugin(String),
    RemovePluginDone(Result<(String, Plugins), String>),
    PluginItemClicked(String),
    EditorAction(text_editor::Action),
    B1nClicked,
    GithubClicked,
    ThemeChanged(Theme),
    ClearMessage(()),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryType {
    Executable,
    DynamicLibrary,
}

impl Display for BinaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executable => write!(f, "Exe"),
            Self::DynamicLibrary => write!(f, "Lib"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellcodeSaveType {
    Local,
    Remote,
}

impl Display for ShellcodeSaveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellcodeSaveType::Local => write!(f, "Local"),
            ShellcodeSaveType::Remote => write!(f, "Remote"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    Darwin,
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Windows => write!(f, "Windows"),
            Platform::Linux => write!(f, "Linux"),
            Platform::Darwin => write!(f, "Darwin"),
        }
    }
}

impl Application for Pumpbin {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;
    type Renderer = Renderer;

    fn new(_flags: Self::Flags) -> (Self, iced::Task<Self::Message>) {
        (Self::default(), Task::none())
    }

    fn title(&self) -> String {
        "PumpBin".into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::ShellcodeSrcChanged(x) => {
                self.shellcode_src = x;
                Task::none()
            }
            Message::ChooseShellcodeClicked => {
                let choose_shellcode = async {
                    let file = AsyncFileDialog::new()
                        .set_directory(home_dir().unwrap_or(".".into()))
                        .set_title("select shellcode file")
                        .pick_file()
                        .await;
                    file.map(|x| x.path().to_path_buf())
                };

                Task::perform(
                    choose_shellcode,
                    match self.shellcode_save_type() {
                        ShellcodeSaveType::Local => Message::ChooseShellcodeDone,
                        ShellcodeSaveType::Remote => Message::EncryptShellcode,
                    },
                )
            }
            Message::ChooseShellcodeDone(x) => {
                if let Some(path) = x {
                    self.shellcode_src = path.to_string_lossy().to_string();
                } else {
                    return self.show_message("Canceled shellcode selection.".into());
                }
                Task::none()
            }
            Message::PlatformChanged(x) => {
                // do nothing if selected this one
                if let Some(selected_platform) = self.selected_platform {
                    if x == selected_platform {
                        return Task::none();
                    }
                }

                self.selected_platform = Some(x);

                // check supported_binary_types

                // unwrap is safe.
                // UI implemented strict restrictions.
                let mut bin_types = Vec::default();
                let platforms = self
                    .plugins()
                    .0
                    .get(self.selected_plugin().unwrap())
                    .unwrap()
                    .platforms();
                match x {
                    Platform::Windows => {
                        if platforms.windows().unwrap().executable().is_some() {
                            bin_types.push(BinaryType::Executable);
                        }
                        if platforms.windows().unwrap().dynamic_library().is_some() {
                            bin_types.push(BinaryType::DynamicLibrary);
                        }
                    }
                    Platform::Linux => {
                        if platforms.linux().unwrap().executable().is_some() {
                            bin_types.push(BinaryType::Executable);
                        }
                        if platforms.linux().unwrap().dynamic_library().is_some() {
                            bin_types.push(BinaryType::DynamicLibrary);
                        }
                    }
                    Platform::Darwin => {
                        if platforms.darwin().unwrap().executable().is_some() {
                            bin_types.push(BinaryType::Executable);
                        }
                        if platforms.darwin().unwrap().dynamic_library().is_some() {
                            bin_types.push(BinaryType::DynamicLibrary);
                        }
                    }
                }

                self.selected_binary_type = None;
                self.supported_binary_types = bin_types;
                Task::none()
            }
            Message::EncryptShellcode(x) => {
                if let Some(path) = x {
                    let encrypt_type = self.encrypt_type().clone();

                    let write_encrypted = async move {
                        let encrypted = encrypt_type
                            .encrypt(&path)
                            .map_err(|_| "Encrypt shellcode failed.")?;

                        let file = AsyncFileDialog::new()
                            .set_directory(desktop_dir().unwrap_or(".".into()))
                            .set_file_name("shellcode.enc")
                            .set_can_create_directories(true)
                            .set_title("save encrypted shellcode")
                            .save_file()
                            .await
                            .ok_or("Canceled saving encrypted shellcode.")?;

                        fs::write(file.path(), encrypted)
                            .map_err(|_| "Write encrypted shellcode failed.")?;

                        Ok(())
                    };
                    Task::perform(write_encrypted, Message::EncryptShellcodeDone)
                } else {
                    self.show_message("Canceled shellcode selection.".into())
                }
            }
            Message::EncryptShellcodeDone(x) => self.show_message(match x {
                Ok(_) => "Saved encrypted shellcode.".into(),
                Err(e) => e,
            }),
            Message::GenerateClicked => {
                // verify path if local mode
                let path = PathBuf::from(self.shellcode_src());
                if self.shellcode_save_type() == ShellcodeSaveType::Local {
                    if path.exists().not() {
                        return self.show_message("Shellcode path not exists.".into());
                    } else if path.is_file().not() {
                        return self.show_message("Shellcode path is not a file.".into());
                    }
                }

                // unwrap is safe.
                // UI implemented strict restrictions.
                let plugin = self
                    .plugins()
                    .0
                    .get(self.selected_plugin().unwrap())
                    .unwrap()
                    .to_owned();
                let encrypt_type = self.encrypt_type().clone();
                let shellcode_save_type = self.shellcode_save_type();
                let shellcode_src = self.shellcode_src().to_owned();

                // get that binary
                let selected_platform = self.selected_platform().unwrap();
                let platforms = plugin.platforms();
                let bin_type = self.selected_binary_type().unwrap();
                let mut bin = match (selected_platform, bin_type) {
                    (Platform::Windows, BinaryType::Executable) => {
                        platforms.windows().unwrap().executable().unwrap().to_vec()
                    }
                    (Platform::Windows, BinaryType::DynamicLibrary) => platforms
                        .windows()
                        .unwrap()
                        .dynamic_library()
                        .unwrap()
                        .to_vec(),
                    (Platform::Linux, BinaryType::Executable) => {
                        platforms.linux().unwrap().executable().unwrap().to_vec()
                    }
                    (Platform::Linux, BinaryType::DynamicLibrary) => platforms
                        .linux()
                        .unwrap()
                        .dynamic_library()
                        .unwrap()
                        .to_vec(),
                    (Platform::Darwin, BinaryType::Executable) => {
                        platforms.darwin().unwrap().executable().unwrap().to_vec()
                    }
                    (Platform::Darwin, BinaryType::DynamicLibrary) => platforms
                        .darwin()
                        .unwrap()
                        .dynamic_library()
                        .unwrap()
                        .to_vec(),
                };

                let generate = async move {
                    // replace pass
                    match &encrypt_type {
                        EncryptType::None => (),
                        EncryptType::Xor(x) => {
                            if let EncryptType::Xor(holder) = plugin.encrypt_type() {
                                let position = memmem::find_iter(bin.as_slice(), holder)
                                    .next()
                                    .ok_or("Can't find the pass placeholder.".to_string())?;

                                bin[position..(position + holder.len())]
                                    .copy_from_slice(x.as_slice());
                            }
                        }
                        EncryptType::AesGcm(x) => {
                            if let EncryptType::AesGcm(holder) = plugin.encrypt_type() {
                                let position =
                                    memmem::find_iter(bin.as_slice(), holder.key_holder())
                                        .next()
                                        .ok_or("Can't find the pass placeholder.".to_string())?;
                                bin[position..(position + holder.key_holder().len())]
                                    .copy_from_slice(x.key_holder());

                                let position =
                                    memmem::find_iter(bin.as_slice(), holder.nonce_holder())
                                        .next()
                                        .ok_or("Can't find the pass placeholder.".to_string())?;
                                bin[position..(position + holder.nonce_holder().len())]
                                    .copy_from_slice(x.nonce_holder());
                            }
                        }
                    }

                    // replace shellcode src
                    match shellcode_save_type {
                        ShellcodeSaveType::Local => {
                            let mut encrypted = encrypt_type
                                .encrypt(&path)
                                .map_err(|_| "Encrypt shellcode failed.".to_string())?;
                            let encrypted_len_bytes =
                                encrypted.len().to_string().as_bytes().to_vec();

                            // unwrap is safe.
                            // UI implemented strict restrictions.
                            let size_holder = plugin.size_holder().unwrap();

                            if encrypted.len() > (plugin.max_len()) {
                                return Err("Shellcode too long.".into());
                            }

                            if encrypted_len_bytes.len() > size_holder.len() {
                                return Err("Shellcode size bytes too long.".into());
                            }

                            let mut size_bytes =
                                vec![b'0'; size_holder.len() - encrypted_len_bytes.len()];

                            size_bytes.extend_from_slice(encrypted_len_bytes.as_slice());

                            // replace size holder
                            let position = memmem::find_iter(bin.as_slice(), size_holder)
                                .next()
                                .ok_or("Can't find the size placeholder.".to_string())?;
                            bin[position..(position + size_holder.len())]
                                .copy_from_slice(size_bytes.as_slice());

                            let mut random: Vec<u8> = iter::repeat(b'0')
                                .take(plugin.max_len() - encrypted.len())
                                .collect();
                            rand::thread_rng().fill_bytes(&mut random);
                            encrypted.extend_from_slice(random.as_slice());

                            // replace shellcode
                            let position = memmem::find_iter(bin.as_slice(), plugin.prefix())
                                .next()
                                .ok_or("Can't find the shellcode prefix.".to_string())?;
                            bin[position..(position + plugin.max_len())]
                                .copy_from_slice(encrypted.as_slice());
                        }
                        ShellcodeSaveType::Remote => {
                            let mut shellcode_src = shellcode_src.as_bytes().to_owned();
                            shellcode_src.push(b'\0');

                            if shellcode_src.len() > plugin.max_len() {
                                return Err("Shellcode url too long.".into());
                            }

                            let mut random: Vec<u8> = iter::repeat(b'0')
                                .take(plugin.max_len() - shellcode_src.len())
                                .collect();
                            rand::thread_rng().fill_bytes(&mut random);
                            shellcode_src.extend_from_slice(random.as_slice());

                            // replace shellcode url
                            let position = memmem::find_iter(bin.as_slice(), plugin.prefix())
                                .next()
                                .ok_or("Can't find the shellcode prefix.".to_string())?;
                            bin[position..(position + plugin.max_len())]
                                .copy_from_slice(shellcode_src.as_slice());
                        }
                    }

                    // write generated binary
                    let file = AsyncFileDialog::new()
                        .set_directory(desktop_dir().unwrap_or(".".into()))
                        .set_file_name("binary.gen")
                        .set_can_create_directories(true)
                        .set_title("save generated binary")
                        .save_file()
                        .await
                        .ok_or("Canceled saving generated binary.".to_string())?;

                    fs::write(file.path(), bin)
                        .map_err(|_| "Write generated binary failed.".to_string())?;

                    Ok(())
                };

                Task::perform(generate, Message::GenerateDone)
            }
            Message::GenerateDone(x) => self.show_message(match x {
                Ok(_) => "Saved generated binary.".into(),
                Err(e) => e,
            }),
            Message::BinaryTypeChanged(x) => {
                self.selected_binary_type = Some(x);
                Task::none()
            }
            Message::AddPluginClicked => {
                let mut plugins = self.plugins().to_owned();

                let add_plugins = async move {
                    let paths: Vec<PathBuf> = AsyncFileDialog::new()
                        .add_filter("b1n", &["b1n"])
                        .set_directory(home_dir().unwrap_or(".".into()))
                        .set_title("select plugin files")
                        .pick_files()
                        .await
                        .map(|x| x.iter().map(|file| file.path().to_owned()).collect())
                        .ok_or("Canceled plugin selection.".to_string())?;

                    let mut success = 0;
                    let mut failed = 0;

                    for path in paths {
                        if let Ok(plugin) = Plugin::reade_plugin(&path) {
                            let plugin_name = plugin.plugin_name().to_owned();

                            plugins.0.insert(plugin_name, plugin);
                            success += 1;
                        } else {
                            failed += 1;
                        }
                    }

                    plugins
                        .uptade_plugins()
                        .map_err(|_| "Uptade plugins failed")?;

                    Ok((success, failed, plugins))
                };

                Task::perform(add_plugins, Message::AddPluginDone)
            }
            Message::AddPluginDone(x) => {
                match x {
                    Ok((success, failed, plugins)) => {
                        // if selected_plugin, reselect this plugin
                        if let Some(selected_plugin) = self.selected_plugin().map(|x| x.to_owned())
                        {
                            // bypass check
                            self.selected_plugin = None;
                            self.update(Message::PluginItemClicked(selected_plugin));
                        }
                        self.plugins = plugins;
                        self.show_message(format!("Added {} plugins, {} failed.", success, failed))
                    }
                    Err(e) => self.show_message(e),
                }
            }
            Message::RemovePlugin(x) => {
                let mut plugins = self.plugins().clone();

                let remove_plugin = async move {
                    plugins
                        .0
                        .remove(&x)
                        .ok_or("Plugin not exists.".to_string())?;
                    plugins
                        .uptade_plugins()
                        .map_err(|_| "Update plugins failed.".to_string())?;

                    Ok((x, plugins))
                };
                Task::perform(remove_plugin, Message::RemovePluginDone)
            }
            Message::RemovePluginDone(x) => match x {
                Ok((plugin_name, plugins)) => {
                    self.plugins = plugins;

                    let mut names: Vec<String> =
                        self.plugins().0.keys().map(|x| x.to_owned()).collect();
                    names.sort();

                    if let Some(name) = names.first() {
                        _ = self.update(Message::PluginItemClicked(name.to_owned()));
                    } else {
                        self.supported_binary_types = Default::default();
                        self.selected_binary_type = None;
                        self.supported_platforms = Default::default();
                        self.selected_platform = None;
                        self.selected_plugin = None;
                        self.shellcode_save_type = ShellcodeSaveType::Local;
                    }
                    self.show_message(format!("Removed plugin {}", plugin_name))
                }
                Err(e) => self.show_message(e),
            },
            Message::PluginItemClicked(x) => {
                // unwrap is safe.
                // UI implemented strict restrictions.
                let plugin = self.plugins.0.get(&x).unwrap();

                if let Some(selected_plugin) = self.selected_plugin() {
                    if plugin.plugin_name() == selected_plugin {
                        // random encryption pass
                        self.random_encrypt_pass();
                        return self.show_message(
                            "Generated new random encryption passwords.".to_string(),
                        );
                    }
                }

                self.selected_plugin = Some(plugin.plugin_name().to_string());

                self.plugin_desc = text_editor::Content::with_text(match plugin.desc() {
                    Some(desc) => desc.as_str(),
                    None => "None",
                });

                if plugin.size_holder().is_none() {
                    self.shellcode_save_type = ShellcodeSaveType::Remote;
                } else {
                    self.shellcode_save_type = ShellcodeSaveType::Local;
                }

                let mut platforms = Vec::default();
                if plugin.platforms().windows().is_some() {
                    platforms.push(Platform::Windows);
                }
                if plugin.platforms().linux().is_some() {
                    platforms.push(Platform::Linux);
                }
                if plugin.platforms().darwin().is_some() {
                    platforms.push(Platform::Darwin);
                }

                self.supported_binary_types = Vec::default();
                self.selected_binary_type = None;
                self.selected_platform = None;
                self.supported_platforms = platforms;

                // random pass
                self.encrypt_type = plugin.encrypt_type().clone();
                self.random_encrypt_pass();

                Task::none()
            }
            Message::EditorAction(x) => {
                match x {
                    text_editor::Action::Edit(_) => (),
                    _ => self.plugin_desc.perform(x),
                }
                Task::none()
            }
            Message::B1nClicked => {
                if open::that(env!("CARGO_PKG_HOMEPAGE")).is_err() {
                    return self.show_message("Open home failed.".into());
                }
                Task::none()
            }
            Message::GithubClicked => {
                if open::that(env!("CARGO_PKG_REPOSITORY")).is_err() {
                    return self.show_message("Open repo failed.".into());
                }
                Task::none()
            }
            Message::ThemeChanged(x) => {
                self.selected_theme = x;
                Task::none()
            }
            Message::ClearMessage(_) => {
                self.message = "Welcome to PumpBin!".to_string();
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let padding = 20;
        let spacing = 20;

        let shellcode_src = row![
            text_input(
                match self.shellcode_save_type() {
                    ShellcodeSaveType::Local => "Shellcode path:",
                    ShellcodeSaveType::Remote => "Shellcode url:",
                },
                &self.shellcode_src
            )
            .on_input(Message::ShellcodeSrcChanged)
            .icon(text_input::Icon {
                font: FONT,
                code_point: '󱓞',
                size: None,
                spacing: 12.0,
                side: text_input::Side::Left,
            }),
            button(match self.shellcode_save_type() {
                ShellcodeSaveType::Local => row![Svg::new(Handle::from_memory(include_bytes!(
                    "../assets/svg/three-dots.svg"
                )))
                .width(20)],
                ShellcodeSaveType::Remote => row![text("󰒃 Encrypt")],
            })
            .on_press(Message::ChooseShellcodeClicked),
        ]
        .spacing(3)
        .align_items(Alignment::Center);

        let pick_list_handle = || pick_list::Handle::Dynamic {
            closed: pick_list::Icon {
                font: FONT,
                code_point: '',
                size: None,
                line_height: text::LineHeight::Relative(1.0),
                shaping: text::Shaping::Basic,
            },
            open: pick_list::Icon {
                font: FONT,
                code_point: '',
                size: None,
                line_height: text::LineHeight::Relative(1.0),
                shaping: text::Shaping::Basic,
            },
        };

        let platform = pick_list(
            self.supported_platforms(),
            self.selected_platform(),
            Message::PlatformChanged,
        )
        .placeholder("Platform")
        .width(100)
        .handle(pick_list_handle());

        let binary_type = pick_list(
            self.supported_binary_types(),
            self.selected_binary_type(),
            Message::BinaryTypeChanged,
        )
        .placeholder("BinType")
        .width(100)
        .handle(pick_list_handle());

        let generate = button(
            row![
                Svg::new(Handle::from_memory(include_bytes!(
                    "../assets/svg/rust-svgrepo-com.svg"
                )))
                .width(20),
                text!("Generate")
            ]
            .spacing(3)
            .align_items(Alignment::Center),
        )
        .on_press_maybe(
            if self.selected_binary_type().is_some() && self.shellcode_src().is_empty().not() {
                Some(Message::GenerateClicked)
            } else {
                None
            },
        );

        let setting_panel = row![shellcode_src, platform, binary_type, generate]
            .spacing(spacing)
            .align_items(Alignment::Center);

        let mut plugin_items = column![]
            .align_items(Alignment::Center)
            .spacing(10)
            .width(Length::Fill)
            .padding(3);

        if self.plugins().0.is_empty() {
            plugin_items = plugin_items.push(
                row![
                    Svg::new(Handle::from_memory(include_bytes!(
                        "../assets/svg/magic-star-svgrepo-com.svg"
                    )))
                    .width(30)
                    .height(30)
                    .style(svg_style::svg_primary_base),
                    text("Did you see that   sign? 󰁂")
                        .color(self.theme().extended_palette().primary.base.color)
                ]
                .spacing(spacing)
                .align_items(Alignment::Center),
            );
        }

        let mut plugin_names: Vec<String> =
            self.plugins().0.keys().map(|x| x.to_string()).collect();
        plugin_names.sort();
        let plugin_names = plugin_names;

        // dynamic push plugin item
        for plugin_name in plugin_names {
            let plugin = match self.plugins().0.get(&plugin_name) {
                Some(x) => x,
                None => continue,
            };

            let item = button(
                column![
                    text!(" {}", plugin_name).width(Length::Fill),
                    row![
                        column![text!(
                            " {}",
                            match plugin.author() {
                                Some(x) => x.to_owned(),
                                None => "None".into(),
                            }
                        )]
                        .width(Length::Fill)
                        .align_items(Alignment::Start),
                        column![row!(
                            text(" ").color(self.theme().extended_palette().primary.base.color),
                            match plugin.platforms().windows() {
                                Some(_) => text(" ")
                                    .color(self.theme().extended_palette().success.base.color),
                                None => text(" ")
                                    .color(self.theme().extended_palette().danger.base.color),
                            },
                            text(" ").color(self.theme().extended_palette().primary.base.color),
                            match plugin.platforms().linux() {
                                Some(_) => text(" ")
                                    .color(self.theme().extended_palette().success.base.color),
                                None => text(" ")
                                    .color(self.theme().extended_palette().danger.base.color),
                            },
                            text(" ").color(self.theme().extended_palette().primary.base.color),
                            match plugin.platforms().darwin() {
                                Some(_) => text(" ")
                                    .color(self.theme().extended_palette().success.base.color),
                                None => text(" ")
                                    .color(self.theme().extended_palette().danger.base.color),
                            }
                        )
                        .align_items(Alignment::Center)]
                        .width(Length::Shrink)
                        .align_items(Alignment::End)
                    ]
                    .align_items(Alignment::Center),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .style(match self.selected_plugin() {
                Some(x) if x == &plugin_name => button_style::selected,
                _ => button_style::unselected,
            })
            .on_press(Message::PluginItemClicked(plugin_name));

            plugin_items = plugin_items.push(item);
        }

        let pumpkin = Svg::new(Handle::from_memory(include_bytes!(
            "../assets/svg/pumpkin-svgrepo-com.svg"
        )))
        .style(|theme: &Theme, _| svg::Style {
            color: Some(theme.extended_palette().background.weak.color),
        });

        let plugin_info_title = |x: &str| {
            text(x.to_owned())
                .size(16)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().primary.base.color),
                })
        };

        let binary_type_some = || {
            text(" ")
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().success.base.color),
                })
                .size(16)
        };

        let binary_type_none = || {
            text(" ")
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                })
                .size(16)
        };

        let plugin_info_panel = column![match self.selected_plugin() {
            Some(plugin_name) => {
                // unwrap is safe.
                // UI implemented strict restrictions.
                let plugin = self.plugins().0.get(plugin_name).unwrap();

                row![
                    column![
                        row![column![
                            plugin_info_title(" Name:"),
                            plugin_info_title(" Author:"),
                            plugin_info_title(" Version:"),
                            plugin_info_title("󰰥 Type:"),
                            plugin_info_title(" MaxLen:"),
                            plugin_info_title("󰒃 Encrypt:"),
                            plugin_info_title(" Windows:"),
                            plugin_info_title(" Linux:"),
                            plugin_info_title(" Darwin:"),
                            plugin_info_title(" Description:"),
                        ]
                        .align_items(Alignment::Start)]
                        .align_items(Alignment::Start),
                        row![pumpkin]
                            .height(Length::Fill)
                            .align_items(Alignment::End),
                    ]
                    .width(Length::FillPortion(1))
                    .align_items(Alignment::Start),
                    column![
                        text(plugin.plugin_name()).size(16),
                        text(match plugin.author() {
                            Some(x) => x.to_string(),
                            None => "None".to_string(),
                        })
                        .size(16),
                        text(match plugin.version() {
                            Some(x) => x.to_string(),
                            None => "None".to_string(),
                        })
                        .size(16),
                        text(match plugin.size_holder().is_none() {
                            true => "Remote",
                            false => "Local",
                        })
                        .size(16),
                        text!("{} Bytes", plugin.max_len()).size(16),
                        text(plugin.encrypt_type().to_string()).size(16),
                        row![
                            text(BinaryType::Executable.to_string()),
                            match plugin.platforms().windows() {
                                Some(bins) if bins.executable().is_some() => binary_type_some(),
                                _ => binary_type_none(),
                            },
                            text(BinaryType::DynamicLibrary.to_string()),
                            match plugin.platforms().windows() {
                                Some(bins) if bins.dynamic_library().is_some() =>
                                    binary_type_some(),
                                _ => binary_type_none(),
                            }
                        ]
                        .spacing(3)
                        .align_items(Alignment::Center),
                        row![
                            text(BinaryType::Executable.to_string()),
                            match plugin.platforms().linux() {
                                Some(bins) if bins.executable().is_some() => binary_type_some(),
                                _ => binary_type_none(),
                            },
                            text(BinaryType::DynamicLibrary.to_string()),
                            match plugin.platforms().linux() {
                                Some(bins) if bins.dynamic_library().is_some() =>
                                    binary_type_some(),
                                _ => binary_type_none(),
                            }
                        ]
                        .spacing(3)
                        .align_items(Alignment::Center),
                        row![
                            text(BinaryType::Executable.to_string()),
                            match plugin.platforms().darwin() {
                                Some(bins) if bins.executable().is_some() => binary_type_some(),
                                _ => binary_type_none(),
                            },
                            text(BinaryType::DynamicLibrary.to_string()),
                            match plugin.platforms().darwin() {
                                Some(bins) if bins.dynamic_library().is_some() =>
                                    binary_type_some(),
                                _ => binary_type_none(),
                            }
                        ]
                        .spacing(3)
                        .align_items(Alignment::Center),
                        text_editor(self.plugin_desc())
                            .padding(10)
                            .height(Length::Fill)
                            .on_action(Message::EditorAction),
                    ]
                    .width(Length::FillPortion(3))
                    .align_items(Alignment::Start)
                ]
                .spacing(spacing)
                .align_items(Alignment::Center)
            }
            None => row![pumpkin],
        }]
        .align_items(Alignment::Start);

        let plugin_list_view = container(
            column![
                Scrollable::with_direction(
                    plugin_items,
                    scrollable::Direction::Vertical(scrollable::Properties::new())
                )
                .width(Length::Fill)
                .height(Length::Fill),
                column![
                    horizontal_rule(0),
                    row![
                        button(
                            Svg::new(Handle::from_memory(include_bytes!(
                                "../assets/svg/iconmonstr-plus-lined.svg"
                            )))
                            .width(20)
                            .height(Length::Fill)
                            .style(svg_style::svg_primary_base)
                        )
                        .on_press(Message::AddPluginClicked)
                        .style(button::text),
                        vertical_rule(0),
                        button(
                            Svg::new(Handle::from_memory(include_bytes!(
                                "../assets/svg/iconmonstr-line-one-horizontal-lined.svg"
                            )))
                            .width(20)
                            .height(Length::Fill)
                            .style(svg_style::svg_primary_base)
                        )
                        .on_press_maybe(
                            self.selected_plugin()
                                .map(|x| Message::RemovePlugin(x.to_string()))
                        )
                        .style(|theme: &Theme, status| {
                            let palette = theme.extended_palette();
                            let mut style = button::text(theme, status);
                            if status == button::Status::Disabled {
                                style.background =
                                    Some(Background::Color(palette.background.weak.color));
                            }

                            style
                        }),
                        vertical_rule(0),
                    ]
                    .width(Length::Fill)
                    .align_items(Alignment::Center),
                ]
                .width(Length::Fill)
                .height(20)
                .align_items(Alignment::Center)
            ]
            .spacing(3)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center),
        )
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style::default().with_border(palette.background.strong.color, 1)
        });

        let plugin_panel = row![
            plugin_info_panel.width(Length::FillPortion(2)),
            plugin_list_view.width(Length::FillPortion(1))
        ]
        .spacing(spacing)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        let message = row![
            text(" ")
                .color(self.theme().extended_palette().primary.base.color)
                .size(25),
            text(&self.message).color(self.theme().extended_palette().primary.base.color)
        ]
        .align_items(Alignment::Center);

        let b1n = button(
            Svg::new(Handle::from_memory(include_bytes!(
                "../assets/svg/house-heart-fill.svg"
            )))
            .width(30)
            .height(30)
            .style(svg_style::svg_primary_base),
        )
        .style(button::text)
        .on_press(Message::B1nClicked);
        let github = button(
            Svg::new(Handle::from_memory(include_bytes!(
                "../assets/svg/github.svg"
            )))
            .width(30)
            .height(30)
            .style(svg_style::svg_primary_base),
        )
        .style(button::text)
        .on_press(Message::GithubClicked);

        let theme_list = pick_list(
            Theme::ALL,
            Some(self.selected_theme.clone()),
            Message::ThemeChanged,
        );

        let footer = column![
            horizontal_rule(0),
            row![
                column![message]
                    .width(Length::FillPortion(1))
                    .align_items(Alignment::Start),
                column![row![b1n, github].align_items(Alignment::Center)]
                    .width(Length::Shrink)
                    .align_items(Alignment::Center),
                column![theme_list]
                    .width(Length::FillPortion(1))
                    .align_items(Alignment::End)
            ]
            .padding([0, padding])
            .align_items(Alignment::Center)
        ]
        .align_items(Alignment::Center);

        let home: Element<_> = column![
            column![setting_panel, plugin_panel]
                .padding(padding)
                .align_items(Alignment::Center)
                .spacing(spacing),
            footer
        ]
        .align_items(Alignment::Center)
        .into();

        home
    }

    fn theme(&self) -> Self::Theme {
        self.selected_theme()
    }
}
