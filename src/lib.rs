pub mod plugin;
mod plugin_system;
pub mod style;
pub mod utils;
pub mod plugin_capnp {
    include!("../capnp/plugin_capnp.rs");
}

use std::{fmt::Display, fs, ops::Not, path::PathBuf};

use anyhow::anyhow;
use dirs::{desktop_dir, home_dir};
use iced::{
    alignment::{Horizontal, Vertical},
    futures::TryFutureExt,
    widget::{
        button, column, container, horizontal_rule, pick_list, row,
        svg::{self, Handle},
        text, text_editor, text_input, vertical_rule, Column, Scrollable, Svg,
    },
    Background, Length, Task, Theme,
};
use plugin::{Plugin, Plugins};
use plugin_system::Pass;
use rfd::{AsyncFileDialog, MessageLevel};
use utils::{message_dialog, JETBRAINS_MONO_FONT};

#[derive(Debug, Clone)]
pub enum Message {
    ShellcodeSrcChanged(String),
    ChooseShellcodeClicked,
    ChooseShellcodeDone(Option<PathBuf>),
    EncryptShellcode(Option<PathBuf>),
    EncryptShellcodeDone(Result<(Vec<Pass>, String), String>),
    PlatformChanged(Platform),
    GenerateClicked,
    GenerateDone(Result<(), String>),
    BinaryTypeChanged(BinaryType),
    AddPluginClicked,
    AddPluginDone(Result<(u32, u32, Plugins), String>),
    RemovePlugin(String),
    RemovePluginDone(Result<Plugins, String>),
    PluginItemClicked(String),
    EditorAction(text_editor::Action),
    B1nClicked,
    GithubClicked,
    ThemeChanged(Theme),
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ShellcodeSaveType {
    #[default]
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

#[derive(Debug)]
pub struct Pumpbin {
    shellcode_src: String,
    shellcode_save_type: ShellcodeSaveType,
    supported_binary_types: Vec<BinaryType>,
    selected_binary_type: Option<BinaryType>,
    supported_platforms: Vec<Platform>,
    selected_platform: Option<Platform>,
    plugins: Plugins,
    selected_plugin: Option<Plugin>,
    plugin_desc: text_editor::Content,
    pass: Vec<Pass>,
    selected_theme: Theme,
}

impl Default for Pumpbin {
    fn default() -> Self {
        Self {
            shellcode_src: Default::default(),
            shellcode_save_type: Default::default(),
            supported_binary_types: Default::default(),
            selected_binary_type: Default::default(),
            supported_platforms: Default::default(),
            selected_platform: Default::default(),
            plugins: Plugins::reade_plugins().unwrap_or_default(),
            selected_plugin: Default::default(),
            plugin_desc: Default::default(),
            pass: Default::default(),
            selected_theme: Theme::CatppuccinMacchiato,
        }
    }
}

impl Pumpbin {
    fn update_supported_binary_types(&mut self, platform: Platform) {
        let bins = self.selected_plugin().unwrap().bins();
        let bin_types = match platform {
            Platform::Windows => bins.windows(),
            Platform::Linux => bins.linux(),
            Platform::Darwin => bins.darwin(),
        }
        .supported_binary_types();

        self.selected_binary_type = None;
        self.supported_binary_types = bin_types;
    }

    fn update_supported_platforms(&mut self, plugin: &Plugin) {
        let platforms = plugin.bins().supported_plaforms();

        self.supported_binary_types = Default::default();
        self.selected_binary_type = Default::default();
        self.supported_platforms = platforms;
        self.selected_platform = Default::default();
    }
}

impl Pumpbin {
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

    pub fn supported_platforms(&self) -> &[Platform] {
        &self.supported_platforms
    }

    pub fn selected_platform(&self) -> Option<Platform> {
        self.selected_platform
    }

    pub fn plugins(&self) -> &Plugins {
        &self.plugins
    }

    pub fn selected_plugin(&self) -> Option<&Plugin> {
        self.selected_plugin.as_ref()
    }

    pub fn plugin_desc(&self) -> &text_editor::Content {
        &self.plugin_desc
    }

    pub fn pass(&self) -> &[Pass] {
        &self.pass
    }

    pub fn selected_theme(&self) -> Theme {
        self.selected_theme.clone()
    }
}

impl Pumpbin {
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ShellcodeSrcChanged(x) => self.shellcode_src = x,
            Message::ChooseShellcodeClicked => {
                let choose_shellcode = async {
                    AsyncFileDialog::new()
                        .set_directory(home_dir().unwrap_or(".".into()))
                        .set_title("Select shellcode file")
                        .pick_file()
                        .await
                        .map(|x| x.path().to_path_buf())
                };

                return Task::perform(
                    choose_shellcode,
                    match self.shellcode_save_type() {
                        ShellcodeSaveType::Local => Message::ChooseShellcodeDone,
                        ShellcodeSaveType::Remote => Message::EncryptShellcode,
                    },
                );
            }
            Message::ChooseShellcodeDone(x) => {
                if let Some(path) = x {
                    self.shellcode_src = path.to_string_lossy().to_string();
                }
            }
            Message::PlatformChanged(x) => {
                // do nothing if selected this platform
                if let Some(selected_platform) = self.selected_platform {
                    if x == selected_platform {
                        return Task::none();
                    }
                }

                self.selected_platform = Some(x);
                self.update_supported_binary_types(x);
            }
            Message::EncryptShellcode(x) => {
                let Some(path) = x else {
                    return Task::none();
                };

                let plugin = self.selected_plugin().unwrap().to_owned();
                let write_encrypted = async move {
                    let output = plugin.plugins().run_encrypt_shellcode(&path)?;
                    let final_shellcode = plugin
                        .plugins()
                        .run_format_encrypted_shellcode(output.encrypted())?;
                    let url = plugin
                        .plugins()
                        .run_upload_final_shellcode_remote(final_shellcode.formated_shellcode())?
                        .url()
                        .to_string();

                    if url.is_empty() {
                        let file = AsyncFileDialog::new()
                            .set_directory(desktop_dir().unwrap_or(".".into()))
                            .set_file_name("shellcode.enc")
                            .set_can_create_directories(true)
                            .set_title("Save encrypted shellcode")
                            .save_file()
                            .await
                            .ok_or(anyhow!("Canceled the saving of encrypted shellcode."))?;

                        fs::write(file.path(), final_shellcode.formated_shellcode())?;
                    }

                    anyhow::Ok((output.pass().to_vec(), url))
                }
                .map_err(|e| e.to_string());

                return Task::perform(write_encrypted, Message::EncryptShellcodeDone);
            }
            Message::EncryptShellcodeDone(x) => {
                match x {
                    Ok((pass, url)) => {
                        self.pass = pass;
                        if url.is_empty().not() {
                            self.shellcode_src = url;
                        }
                        message_dialog("Encrypted shellcode done.".into(), MessageLevel::Info)
                    }
                    Err(e) => message_dialog(e, MessageLevel::Error),
                };
            }
            Message::GenerateClicked => {
                // unwrap is safe.
                // UI implemented strict restrictions.
                let plugin = self.selected_plugin().unwrap().to_owned();
                let shellcode_src = self.shellcode_src().to_owned();
                let pass = self.pass().to_vec();

                // get that binary
                let mut bin = plugin.bins().get_that_binary(
                    self.selected_platform().unwrap(),
                    self.selected_binary_type().unwrap(),
                );

                let generate = async move {
                    plugin.replace_binary(&mut bin, shellcode_src, pass)?;

                    // write generated binary
                    let file = AsyncFileDialog::new()
                        .set_directory(desktop_dir().unwrap_or(".".into()))
                        .set_file_name("binary.gen")
                        .set_can_create_directories(true)
                        .set_title("Save generated binary")
                        .save_file()
                        .await
                        .ok_or(anyhow!("Canceled the saving of the generated binary."))?;

                    fs::write(file.path(), bin)?;

                    anyhow::Ok(())
                }
                .map_err(|e| e.to_string());

                return Task::perform(generate, Message::GenerateDone);
            }
            Message::GenerateDone(x) => {
                match x {
                    Ok(_) => message_dialog("Generate done.".into(), MessageLevel::Info),
                    Err(e) => message_dialog(e, MessageLevel::Error),
                };
            }
            Message::BinaryTypeChanged(x) => self.selected_binary_type = Some(x),
            Message::AddPluginClicked => {
                let mut plugins = self.plugins().clone();

                let add_plugins = async move {
                    let files = AsyncFileDialog::new()
                        .add_filter("b1n", &["b1n"])
                        .set_directory(home_dir().unwrap_or(".".into()))
                        .set_title("Select plugin files")
                        .pick_files()
                        .await
                        .ok_or(anyhow!("Canceled the selection of plugin files."))?;

                    let mut success = 0;
                    let mut failed = 0;

                    for path in files.iter().map(|x| x.path()) {
                        let Ok(buf) = fs::read(path) else {
                            failed += 1;
                            continue;
                        };
                        if let Ok(plugin) = Plugin::decode_from_slice(buf.as_slice()) {
                            let plugin_name = plugin.info().plugin_name();

                            plugins.insert(plugin_name.to_string(), buf);
                            success += 1;
                        } else {
                            failed += 1;
                        }
                    }

                    plugins.uptade_plugins()?;
                    anyhow::Ok((success, failed, plugins))
                }
                .map_err(|e| e.to_string());

                return Task::perform(add_plugins, Message::AddPluginDone);
            }
            Message::AddPluginDone(x) => {
                match x {
                    Ok((success, failed, plugins)) => {
                        // if selected_plugin, reselect this plugin
                        if let Some(selected_plugin) = self.selected_plugin() {
                            let plugin_name = selected_plugin.info().plugin_name().to_owned();

                            // bypass check
                            self.selected_plugin = None;
                            self.update(Message::PluginItemClicked(plugin_name));
                        }
                        self.plugins = plugins;
                        message_dialog(
                            format!("Added {} plugins, {} failed.", success, failed),
                            MessageLevel::Info,
                        );
                    }
                    Err(e) => {
                        message_dialog(e, MessageLevel::Error);
                    }
                }
            }
            Message::RemovePlugin(x) => {
                let mut plugins = self.plugins().clone();

                let remove_plugin = async move {
                    plugins.remove(&x);
                    plugins.uptade_plugins()?;

                    anyhow::Ok(plugins)
                }
                .map_err(|e| e.to_string());

                return Task::perform(remove_plugin, Message::RemovePluginDone);
            }
            Message::RemovePluginDone(x) => {
                match x {
                    Ok(plugins) => {
                        self.plugins = plugins;

                        if let Some(name) = self.plugins().get_sorted_names().first() {
                            _ = self.update(Message::PluginItemClicked(name.to_owned()));
                        } else {
                            self.supported_binary_types = Default::default();
                            self.selected_binary_type = None;
                            self.supported_platforms = Default::default();
                            self.selected_platform = None;
                            self.selected_plugin = None;
                            self.shellcode_save_type = ShellcodeSaveType::Local;
                        }
                    }
                    Err(e) => {
                        message_dialog(e, MessageLevel::Error);
                    }
                };
            }
            Message::PluginItemClicked(x) => {
                // unwrap is safe.
                // UI implemented strict restrictions.
                let plugin = self.plugins().get(&x).unwrap();

                if let Some(selected_plugin) = self.selected_plugin() {
                    if plugin.info().plugin_name() == selected_plugin.info().plugin_name() {
                        return Task::none();
                    }
                }

                self.selected_plugin = Some(plugin.clone());
                self.plugin_desc = text_editor::Content::with_text(plugin.info().desc());

                if plugin.replace().size_holder().is_some() {
                    self.shellcode_save_type = ShellcodeSaveType::Local;
                } else {
                    self.shellcode_save_type = ShellcodeSaveType::Remote;
                }

                self.update_supported_platforms(&plugin);
            }
            Message::EditorAction(x) => match x {
                text_editor::Action::Edit(_) => (),
                _ => self.plugin_desc.perform(x),
            },
            Message::B1nClicked => {
                if open::that(env!("CARGO_PKG_HOMEPAGE")).is_err() {
                    message_dialog("Open home failed.".into(), MessageLevel::Error);
                }
            }
            Message::GithubClicked => {
                if open::that(env!("CARGO_PKG_REPOSITORY")).is_err() {
                    message_dialog("Open repo failed.".into(), MessageLevel::Error);
                }
            }
            Message::ThemeChanged(x) => self.selected_theme = x,
        }

        Task::none()
    }

    pub fn view(&self) -> Column<Message> {
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
                font: JETBRAINS_MONO_FONT,
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
        .align_y(Vertical::Center);

        let pick_list_handle = || pick_list::Handle::Dynamic {
            closed: pick_list::Icon {
                font: JETBRAINS_MONO_FONT,
                code_point: '',
                size: None,
                line_height: text::LineHeight::Relative(1.0),
                shaping: text::Shaping::Basic,
            },
            open: pick_list::Icon {
                font: JETBRAINS_MONO_FONT,
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
            .align_y(Vertical::Center),
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
            .align_y(Vertical::Center);

        let mut plugin_items = column![]
            .align_x(Horizontal::Center)
            .spacing(10)
            .width(Length::Fill)
            .padding(3);

        if self.plugins().is_empty() {
            plugin_items = plugin_items.push(
                row![
                    Svg::new(Handle::from_memory(include_bytes!(
                        "../assets/svg/magic-star-svgrepo-com.svg"
                    )))
                    .width(30)
                    .height(30)
                    .style(style::svg::svg_primary_base),
                    text("Did you see that  sign? 󰁂")
                        .color(self.theme().extended_palette().primary.base.color)
                ]
                .spacing(spacing)
                .align_y(Vertical::Center),
            );
        }

        let plugin_names = self.plugins().get_sorted_names();

        // dynamic push plugin item
        for plugin_name in plugin_names {
            let plugin = match self.plugins().get(&plugin_name) {
                Ok(x) => x,
                Err(_) => continue,
            };

            let item = button(
                column![
                    text!(" {}", plugin_name).width(Length::Fill),
                    row![
                        column![text!(" {}", plugin.info().author())]
                            .width(Length::Fill)
                            .align_x(Horizontal::Left),
                        column![row!(
                            text(" ").color(self.theme().extended_palette().primary.base.color),
                            if plugin.bins().windows().is_platform_supported() {
                                text(" ").color(self.theme().extended_palette().success.base.color)
                            } else {
                                text(" ").color(self.theme().extended_palette().danger.base.color)
                            },
                            text(" ").color(self.theme().extended_palette().primary.base.color),
                            if plugin.bins().linux().is_platform_supported() {
                                text(" ").color(self.theme().extended_palette().success.base.color)
                            } else {
                                text(" ").color(self.theme().extended_palette().danger.base.color)
                            },
                            text(" ").color(self.theme().extended_palette().primary.base.color),
                            if plugin.bins().darwin().is_platform_supported() {
                                text(" ").color(self.theme().extended_palette().success.base.color)
                            } else {
                                text(" ").color(self.theme().extended_palette().danger.base.color)
                            }
                        )
                        .align_y(Vertical::Center)]
                        .width(Length::Shrink)
                        .align_x(Horizontal::Right)
                    ]
                    .align_y(Vertical::Center),
                ]
                .align_x(Horizontal::Center),
            )
            .width(Length::Fill)
            .style(match self.selected_plugin() {
                Some(x) if x.info().plugin_name() == plugin_name => style::button::selected,
                _ => style::button::unselected,
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
            Some(plugin) => {
                row![
                    column![
                        row![column![
                            plugin_info_title(" Name:"),
                            plugin_info_title(" Author:"),
                            plugin_info_title(" Version:"),
                            plugin_info_title("󰰥 Type:"),
                            plugin_info_title(" MaxLen:"),
                            plugin_info_title(" Windows:"),
                            plugin_info_title(" Linux:"),
                            plugin_info_title(" Darwin:"),
                            plugin_info_title(" Description:"),
                        ]
                        .align_x(Horizontal::Left)]
                        .align_y(Vertical::Top),
                        row![pumpkin].height(Length::Fill).align_y(Vertical::Bottom),
                    ]
                    .width(Length::FillPortion(1))
                    .align_x(Horizontal::Left),
                    column![
                        text(plugin.info().plugin_name()).size(16),
                        text(plugin.info().author()).size(16),
                        text(plugin.info().version()).size(16),
                        text(match plugin.replace().size_holder().is_none() {
                            true => "Remote",
                            false => "Local",
                        })
                        .size(16),
                        text!("{} Bytes", plugin.replace().max_len()).size(16),
                        row![
                            text(BinaryType::Executable.to_string()),
                            {
                                let bins = plugin.bins().windows();
                                if bins.executable().is_some() {
                                    binary_type_some()
                                } else {
                                    binary_type_none()
                                }
                            },
                            text(BinaryType::DynamicLibrary.to_string()),
                            {
                                let bins = plugin.bins().windows();
                                if bins.dynamic_library().is_some() {
                                    binary_type_some()
                                } else {
                                    binary_type_none()
                                }
                            }
                        ]
                        .spacing(3)
                        .align_y(Vertical::Center),
                        row![
                            text(BinaryType::Executable.to_string()),
                            {
                                let bins = plugin.bins().linux();
                                if bins.executable().is_some() {
                                    binary_type_some()
                                } else {
                                    binary_type_none()
                                }
                            },
                            text(BinaryType::DynamicLibrary.to_string()),
                            {
                                let bins = plugin.bins().linux();
                                if bins.dynamic_library().is_some() {
                                    binary_type_some()
                                } else {
                                    binary_type_none()
                                }
                            }
                        ]
                        .spacing(3)
                        .align_y(Vertical::Center),
                        row![
                            text(BinaryType::Executable.to_string()),
                            {
                                let bins = plugin.bins().darwin();
                                if bins.executable().is_some() {
                                    binary_type_some()
                                } else {
                                    binary_type_none()
                                }
                            },
                            text(BinaryType::DynamicLibrary.to_string()),
                            {
                                let bins = plugin.bins().darwin();
                                if bins.dynamic_library().is_some() {
                                    binary_type_some()
                                } else {
                                    binary_type_none()
                                }
                            }
                        ]
                        .spacing(3)
                        .align_y(Vertical::Center),
                        text_editor(self.plugin_desc())
                            .padding(10)
                            .height(Length::Fill)
                            .on_action(Message::EditorAction),
                    ]
                    .width(Length::FillPortion(3))
                    .align_x(Horizontal::Left)
                ]
                .spacing(spacing)
                .align_y(Vertical::Center)
            }
            None => row![pumpkin],
        }]
        .align_x(Horizontal::Left);

        let plugin_list_view = container(
            column![
                Scrollable::new(plugin_items)
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
                            .style(style::svg::svg_primary_base)
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
                            .style(style::svg::svg_primary_base)
                        )
                        .on_press_maybe(
                            self.selected_plugin()
                                .map(|x| Message::RemovePlugin(x.info().plugin_name().to_string()))
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
                    .align_y(Vertical::Center),
                ]
                .width(Length::Fill)
                .height(20)
                .align_x(Horizontal::Center)
            ]
            .spacing(3)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center),
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
        .align_y(Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        let version = text(format!("PumpBin  v{}", env!("CARGO_PKG_VERSION")))
            .color(self.theme().extended_palette().primary.base.color);

        let b1n = button(
            Svg::new(Handle::from_memory(include_bytes!(
                "../assets/svg/house-heart-fill.svg"
            )))
            .width(30)
            .height(30)
            .style(style::svg::svg_primary_base),
        )
        .style(button::text)
        .on_press(Message::B1nClicked);
        let github = button(
            Svg::new(Handle::from_memory(include_bytes!(
                "../assets/svg/github.svg"
            )))
            .width(30)
            .height(30)
            .style(style::svg::svg_primary_base),
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
                column![version]
                    .width(Length::Fill)
                    .align_x(Horizontal::Left),
                column![row![b1n, github].align_y(Vertical::Center)]
                    .width(Length::Shrink)
                    .align_x(Horizontal::Center),
                column![theme_list]
                    .width(Length::Fill)
                    .align_x(Horizontal::Right)
            ]
            .padding([0, padding])
            .align_y(Vertical::Center)
        ]
        .align_x(Horizontal::Center);

        let home = column![
            column![setting_panel, plugin_panel]
                .padding(padding)
                .align_x(Horizontal::Center)
                .spacing(spacing),
            footer
        ]
        .align_x(Horizontal::Center);

        home
    }

    pub fn theme(&self) -> Theme {
        self.selected_theme()
    }
}
