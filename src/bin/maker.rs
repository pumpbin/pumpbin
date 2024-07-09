#![windows_subsystem = "windows"]

use std::{fs, ops::Not, path::PathBuf};

use anyhow::{anyhow, bail};
use dirs::{desktop_dir, home_dir};
use iced::{
    application,
    futures::TryFutureExt,
    widget::{
        button, column, horizontal_rule, pick_list, radio, row, svg::Handle, text, text_editor,
        text_input, Column, Svg,
    },
    Alignment, Length, Size, Task, Theme,
};
use pumpbin::{
    plugin::{Plugin, PluginInfo, PluginReplace},
    utils::{self, error_dialog, message_dialog},
};
use pumpbin::{style, ShellcodeSaveType};
use rfd::{AsyncFileDialog, MessageLevel};

fn main() {
    if let Err(e) = try_main() {
        error_dialog(e);
    }
}

fn try_main() -> anyhow::Result<()> {
    let size = Size::new(1200.0, 800.0);

    let mut window_settings = utils::window_settings();
    window_settings.size = size;
    window_settings.min_size = Some(size);

    application("PumpBin Maker", Maker::update, Maker::view)
        .settings(utils::settings())
        .window(window_settings)
        .theme(Maker::theme)
        .run()?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum ChooseFileType {
    WindowsExe,
    WindowsLib,
    LinuxExe,
    LinuxLib,
    DarwinExe,
    DarwinLib,
    EncryptShellcodePlugin,
    FormatEncryptedShellcodePlugin,
    FormatUrlRemote,
    UploadFinalShellcodeRemote,
}

#[derive(Debug, Clone)]
enum MakerMessage {
    PluginNameChanged(String),
    AuthorChanged(String),
    VersionChanged(String),
    SrcPrefixChanged(String),
    MaxLenChanged(String),
    ShellcodeSaveTypeChanged(ShellcodeSaveType),
    SizeHolderChanged(String),
    WindowsExeChanged(String),
    WindowsLibChanged(String),
    LinuxExeChanged(String),
    LinuxLibChanged(String),
    DarwinExeChanged(String),
    DarwinLibChanged(String),
    EncryptShllcodePluginChanged(String),
    FormatEncryptedShellcodePluginChanged(String),
    FormatUrlRemotePluginChanged(String),
    UploadFinalShellcodeRemotePluginChanged(String),
    DescAction(text_editor::Action),
    GenerateClicked,
    GenerateDone(Result<(), String>),
    ChooseFileClicked(ChooseFileType),
    ChooseFileDone((Option<String>, ChooseFileType)),
    B1nClicked,
    GithubClicked,
    ThemeChanged(Theme),
}

#[derive(Debug)]
struct Maker {
    plugin_name: String,
    author: String,
    version: String,
    src_prefix: String,
    max_len: String,
    shellcode_save_type: ShellcodeSaveType,
    size_holder: String,
    windows_exe: String,
    windows_lib: String,
    linux_exe: String,
    linux_lib: String,
    darwin_exe: String,
    darwin_lib: String,
    encrypt_shellcode_plugin: String,
    format_encrypted_shellcode_plugin: String,
    format_url_remote_plugin: String,
    upload_final_shellcode_remote_plugin: String,
    desc: text_editor::Content,
    pumpbin_version: String,
    selected_theme: Theme,
}

impl Maker {
    fn check_generate(&self) -> anyhow::Result<()> {
        if self.plugin_name.is_empty() {
            bail!("Plugin Name is empty.");
        }

        if self.src_prefix.is_empty() {
            bail!("Prefix is empty.");
        }

        let max_len = self.max_len();
        if max_len.is_empty() {
            bail!("Max Len is empty.");
        }

        if max_len.parse::<usize>().is_err() {
            bail!("Max Len numeric only.");
        };

        if let ShellcodeSaveType::Local = self.shellcode_save_type() {
            if self.size_holder().is_empty() {
                bail!("Size Holder is empty.");
            }
        };

        anyhow::Ok(())
    }
}

impl Default for Maker {
    fn default() -> Self {
        Self {
            plugin_name: Default::default(),
            author: Default::default(),
            version: Default::default(),
            src_prefix: Default::default(),
            max_len: Default::default(),
            shellcode_save_type: Default::default(),
            size_holder: Default::default(),
            windows_exe: Default::default(),
            windows_lib: Default::default(),
            linux_exe: Default::default(),
            linux_lib: Default::default(),
            darwin_exe: Default::default(),
            darwin_lib: Default::default(),
            encrypt_shellcode_plugin: Default::default(),
            format_encrypted_shellcode_plugin: Default::default(),
            format_url_remote_plugin: Default::default(),
            upload_final_shellcode_remote_plugin: Default::default(),
            desc: text_editor::Content::new(),
            pumpbin_version: env!("CARGO_PKG_VERSION").into(),
            selected_theme: Theme::CatppuccinMacchiato,
        }
    }
}

impl Maker {
    fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    fn author(&self) -> &str {
        &self.author
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn src_prefix(&self) -> &str {
        &self.src_prefix
    }

    fn max_len(&self) -> &str {
        &self.max_len
    }

    fn shellcode_save_type(&self) -> ShellcodeSaveType {
        self.shellcode_save_type
    }

    fn size_holder(&self) -> &str {
        &self.size_holder
    }

    fn windows_exe(&self) -> &str {
        &self.windows_exe
    }

    fn windows_lib(&self) -> &str {
        &self.windows_lib
    }

    fn linux_exe(&self) -> &str {
        &self.linux_exe
    }

    fn linux_lib(&self) -> &str {
        &self.linux_lib
    }

    fn darwin_exe(&self) -> &str {
        &self.darwin_exe
    }

    fn darwin_lib(&self) -> &str {
        &self.darwin_lib
    }

    fn encrypt_shellcode_plugin(&self) -> &str {
        &self.encrypt_shellcode_plugin
    }

    fn format_encrypted_shellcode_plugin(&self) -> &str {
        &self.format_encrypted_shellcode_plugin
    }

    fn format_url_remote_plugin(&self) -> &str {
        &self.format_url_remote_plugin
    }

    fn upload_final_shellcode_remote_plugin(&self) -> &str {
        &self.upload_final_shellcode_remote_plugin
    }

    fn desc(&self) -> &text_editor::Content {
        &self.desc
    }

    fn desc_mut(&mut self) -> &mut text_editor::Content {
        &mut self.desc
    }

    fn selected_theme(&self) -> Theme {
        self.selected_theme.clone()
    }

    fn pumpbin_version(&self) -> &str {
        &self.pumpbin_version
    }
}

impl Maker {
    pub fn update(&mut self, message: MakerMessage) -> iced::Task<MakerMessage> {
        match message {
            MakerMessage::PluginNameChanged(x) => self.plugin_name = x,
            MakerMessage::AuthorChanged(x) => self.author = x,
            MakerMessage::VersionChanged(x) => self.version = x,
            MakerMessage::SrcPrefixChanged(x) => self.src_prefix = x,
            MakerMessage::MaxLenChanged(x) => self.max_len = x,
            MakerMessage::ShellcodeSaveTypeChanged(x) => self.shellcode_save_type = x,
            MakerMessage::SizeHolderChanged(x) => self.size_holder = x,
            MakerMessage::WindowsExeChanged(x) => self.windows_exe = x,
            MakerMessage::WindowsLibChanged(x) => self.windows_lib = x,
            MakerMessage::LinuxExeChanged(x) => self.linux_exe = x,
            MakerMessage::LinuxLibChanged(x) => self.linux_lib = x,
            MakerMessage::DarwinExeChanged(x) => self.darwin_exe = x,
            MakerMessage::DarwinLibChanged(x) => self.darwin_lib = x,
            MakerMessage::EncryptShllcodePluginChanged(x) => self.encrypt_shellcode_plugin = x,
            MakerMessage::FormatEncryptedShellcodePluginChanged(x) => {
                self.format_encrypted_shellcode_plugin = x
            }
            MakerMessage::FormatUrlRemotePluginChanged(x) => self.format_url_remote_plugin = x,
            MakerMessage::UploadFinalShellcodeRemotePluginChanged(x) => {
                self.upload_final_shellcode_remote_plugin = x
            }
            MakerMessage::DescAction(x) => self.desc_mut().perform(x),
            MakerMessage::GenerateClicked => {
                if let Err(e) = self.check_generate() {
                    message_dialog(e.to_string(), MessageLevel::Error);
                    return Task::none();
                }

                let mut plugin = Plugin {
                    version: self.pumpbin_version().to_string(),
                    info: PluginInfo {
                        plugin_name: self.plugin_name().to_string(),
                        author: {
                            let author = self.author().to_string();
                            if author.is_empty() {
                                "None".to_string()
                            } else {
                                author
                            }
                        },
                        version: {
                            let version = self.version().to_string();
                            if version.is_empty() {
                                "None".to_string()
                            } else {
                                version
                            }
                        },
                        desc: {
                            let desc = self.desc().text();
                            if desc.is_empty() {
                                "None".to_string()
                            } else {
                                desc
                            }
                        },
                    },
                    replace: PluginReplace {
                        src_prefix: self.src_prefix().as_bytes().to_vec(),
                        size_holder: match self.shellcode_save_type() {
                            ShellcodeSaveType::Local => {
                                Some(self.size_holder().as_bytes().to_vec())
                            }
                            ShellcodeSaveType::Remote => None,
                        },
                        max_len: self.max_len().parse().unwrap(),
                    },
                    ..Default::default()
                };

                let paths: Vec<(String, ChooseFileType)> = vec![
                    (self.windows_exe(), ChooseFileType::WindowsExe),
                    (self.windows_lib(), ChooseFileType::WindowsLib),
                    (self.linux_exe(), ChooseFileType::LinuxExe),
                    (self.linux_lib(), ChooseFileType::LinuxLib),
                    (self.darwin_exe(), ChooseFileType::DarwinExe),
                    (self.darwin_lib(), ChooseFileType::DarwinLib),
                    (
                        self.encrypt_shellcode_plugin(),
                        ChooseFileType::EncryptShellcodePlugin,
                    ),
                    (
                        self.format_encrypted_shellcode_plugin(),
                        ChooseFileType::FormatEncryptedShellcodePlugin,
                    ),
                    (
                        self.format_url_remote_plugin(),
                        ChooseFileType::FormatUrlRemote,
                    ),
                    (
                        self.upload_final_shellcode_remote_plugin(),
                        ChooseFileType::UploadFinalShellcodeRemote,
                    ),
                ]
                .into_iter()
                .map(|(x, y)| (x.to_string(), y))
                .collect();

                let make_plugin = async move {
                    for (path_str, file_type) in paths {
                        if path_str.is_empty().not() {
                            let path = PathBuf::from(path_str);
                            let data = fs::read(path)?;
                            let bin = match file_type {
                                ChooseFileType::WindowsExe => plugin.bins.windows.executable_mut(),
                                ChooseFileType::WindowsLib => {
                                    plugin.bins.windows.dynamic_library_mut()
                                }
                                ChooseFileType::LinuxExe => plugin.bins.linux.executable_mut(),
                                ChooseFileType::LinuxLib => plugin.bins.linux.dynamic_library_mut(),
                                ChooseFileType::DarwinExe => plugin.bins.darwin.executable_mut(),
                                ChooseFileType::DarwinLib => {
                                    plugin.bins.darwin.dynamic_library_mut()
                                }
                                ChooseFileType::EncryptShellcodePlugin => {
                                    plugin.plugins.encrypt_shellcode_mut()
                                }
                                ChooseFileType::FormatEncryptedShellcodePlugin => {
                                    plugin.plugins.format_encrypted_shellcode_mut()
                                }
                                ChooseFileType::FormatUrlRemote => {
                                    plugin.plugins.format_url_remote_mut()
                                }
                                ChooseFileType::UploadFinalShellcodeRemote => {
                                    plugin.plugins.upload_final_shellcode_remote_mut()
                                }
                            };
                            *bin = Some(data);
                        }
                    }

                    let file = AsyncFileDialog::new()
                        .set_directory(desktop_dir().unwrap_or(".".into()))
                        .set_file_name(format!("{}.b1n", plugin.info().plugin_name()))
                        .set_can_create_directories(true)
                        .set_title("Save generated plugin")
                        .save_file()
                        .await
                        .ok_or(anyhow!("Canceled the saving of the generated plugin."))?;

                    fs::write(file.path(), plugin.encode_to_vec()?)?;

                    anyhow::Ok(())
                }
                .map_err(|e| e.to_string());

                return Task::perform(make_plugin, MakerMessage::GenerateDone);
            }
            MakerMessage::GenerateDone(x) => {
                match x {
                    Ok(_) => message_dialog("Generate done.".to_string(), MessageLevel::Info),
                    Err(e) => message_dialog(e, MessageLevel::Error),
                };
            }
            MakerMessage::ChooseFileClicked(x) => {
                let choose_file = async move {
                    let file = AsyncFileDialog::new()
                        .set_directory(home_dir().unwrap_or(".".into()))
                        .set_title("Choose file")
                        .pick_file()
                        .await
                        .map(|x| x.path().to_string_lossy().to_string());

                    (file, x)
                };

                return Task::perform(choose_file, MakerMessage::ChooseFileDone);
            }
            MakerMessage::ChooseFileDone((path, choose_type)) => {
                if let Some(path) = path {
                    match choose_type {
                        ChooseFileType::WindowsExe => self.windows_exe = path,
                        ChooseFileType::WindowsLib => self.windows_lib = path,
                        ChooseFileType::LinuxExe => self.linux_exe = path,
                        ChooseFileType::LinuxLib => self.linux_lib = path,
                        ChooseFileType::DarwinExe => self.darwin_exe = path,
                        ChooseFileType::DarwinLib => self.darwin_lib = path,
                        ChooseFileType::EncryptShellcodePlugin => {
                            self.encrypt_shellcode_plugin = path
                        }
                        ChooseFileType::FormatEncryptedShellcodePlugin => {
                            self.format_encrypted_shellcode_plugin = path
                        }
                        ChooseFileType::FormatUrlRemote => self.format_url_remote_plugin = path,
                        ChooseFileType::UploadFinalShellcodeRemote => {
                            self.upload_final_shellcode_remote_plugin = path
                        }
                    }
                }
            }
            MakerMessage::B1nClicked => {
                if open::that(env!("CARGO_PKG_HOMEPAGE")).is_err() {
                    message_dialog("Open home failed.".into(), MessageLevel::Error);
                }
            }
            MakerMessage::GithubClicked => {
                if open::that(env!("CARGO_PKG_REPOSITORY")).is_err() {
                    message_dialog("Open repo failed.".into(), MessageLevel::Error);
                }
            }
            MakerMessage::ThemeChanged(x) => self.selected_theme = x,
        }

        Task::none()
    }

    pub fn view(&self) -> Column<MakerMessage> {
        let choose_button = || {
            button(
                Svg::new(Handle::from_memory(include_bytes!(
                    "../../assets/svg/three-dots.svg"
                )))
                .width(20),
            )
        };

        let maker = column![
            row![
                column![
                    text("Plugin Name"),
                    text_input("", self.plugin_name()).on_input(MakerMessage::PluginNameChanged),
                ]
                .align_items(Alignment::Start),
                column![
                    text("Author"),
                    text_input("", self.author()).on_input(MakerMessage::AuthorChanged),
                ]
                .align_items(Alignment::Start),
                column![
                    text("Version"),
                    text_input("", self.version()).on_input(MakerMessage::VersionChanged),
                ]
                .align_items(Alignment::Start),
                column![
                    text("Prefix"),
                    text_input("", self.src_prefix()).on_input(MakerMessage::SrcPrefixChanged),
                ]
                .align_items(Alignment::Start),
                column![
                    text("Max Len"),
                    text_input("", self.max_len()).on_input(MakerMessage::MaxLenChanged),
                ]
                .align_items(Alignment::Start),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
            column![
                text("Type"),
                row![
                    radio(
                        ShellcodeSaveType::Local.to_string(),
                        ShellcodeSaveType::Local,
                        Some(self.shellcode_save_type()),
                        MakerMessage::ShellcodeSaveTypeChanged
                    ),
                    radio(
                        ShellcodeSaveType::Remote.to_string(),
                        ShellcodeSaveType::Remote,
                        Some(self.shellcode_save_type()),
                        MakerMessage::ShellcodeSaveTypeChanged
                    )
                ]
                .push_maybe(match self.shellcode_save_type() {
                    ShellcodeSaveType::Local => Some(
                        row![
                            text("Size Holder: "),
                            text_input("", self.size_holder())
                                .on_input(MakerMessage::SizeHolderChanged)
                        ]
                        .align_items(Alignment::Center)
                    ),
                    ShellcodeSaveType::Remote => None,
                })
                .align_items(Alignment::Center)
                .height(30)
                .spacing(20)
            ]
            .align_items(Alignment::Start),
            column![
                text("Windows"),
                row![
                    text("Exe:"),
                    row![text_input("", self.windows_exe())
                        .on_input(MakerMessage::WindowsExeChanged)],
                    choose_button()
                        .on_press(MakerMessage::ChooseFileClicked(ChooseFileType::WindowsExe)),
                    text("Lib:"),
                    text_input("", self.windows_lib()).on_input(MakerMessage::WindowsLibChanged),
                    choose_button()
                        .on_press(MakerMessage::ChooseFileClicked(ChooseFileType::WindowsLib)),
                ]
                .align_items(Alignment::Center)
                .spacing(10)
            ]
            .align_items(Alignment::Start),
            column![
                text("Linux"),
                row![
                    text("Exe:"),
                    row![text_input("", self.linux_exe()).on_input(MakerMessage::LinuxExeChanged)],
                    choose_button()
                        .on_press(MakerMessage::ChooseFileClicked(ChooseFileType::LinuxExe)),
                    text("Lib:"),
                    text_input("", self.linux_lib()).on_input(MakerMessage::LinuxLibChanged),
                    choose_button()
                        .on_press(MakerMessage::ChooseFileClicked(ChooseFileType::LinuxLib)),
                ]
                .align_items(Alignment::Center)
                .spacing(10)
            ]
            .align_items(Alignment::Start),
            column![
                    text("Darwin"),
                    row![
                        text("Exe:"),
                        row![text_input("", self.darwin_exe())
                            .on_input(MakerMessage::DarwinExeChanged)],
                        choose_button()
                            .on_press(MakerMessage::ChooseFileClicked(ChooseFileType::DarwinExe)),
                        text("Lib:"),
                        text_input("", self.darwin_lib()).on_input(MakerMessage::DarwinLibChanged),
                        choose_button()
                            .on_press(MakerMessage::ChooseFileClicked(ChooseFileType::DarwinLib)),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(10)
                ]
            .align_items(Alignment::Start),
            row![
                column![column![
                    text("Encrypt Shellcode Plug-in"),
                    row![
                        text_input("", self.encrypt_shellcode_plugin())
                            .on_input(MakerMessage::EncryptShllcodePluginChanged),
                        choose_button().on_press(MakerMessage::ChooseFileClicked(
                            ChooseFileType::EncryptShellcodePlugin
                        ))
                    ]
                    .align_items(Alignment::Center)
                    .spacing(10),
                ]
                .align_items(Alignment::Start)]
                .push_maybe(match self.shellcode_save_type() {
                    ShellcodeSaveType::Local => None,
                    ShellcodeSaveType::Remote => Some(column![
                        text("Format Url Remote Plug-in"),
                        row![
                            text_input("", self.format_url_remote_plugin())
                                .on_input(MakerMessage::FormatUrlRemotePluginChanged),
                            choose_button().on_press(MakerMessage::ChooseFileClicked(
                                ChooseFileType::FormatUrlRemote
                            ))
                        ]
                        .align_items(Alignment::Center)
                        .spacing(10)
                    ]),
                })
                .width(Length::FillPortion(1))
                .align_items(Alignment::Center),
                column![column![
                    text("Format Encrypted Shellcode Plug-in"),
                    row![
                        text_input("", self.format_encrypted_shellcode_plugin())
                            .on_input(MakerMessage::FormatEncryptedShellcodePluginChanged),
                        choose_button().on_press(MakerMessage::ChooseFileClicked(
                            ChooseFileType::FormatEncryptedShellcodePlugin
                        ))
                    ]
                    .align_items(Alignment::Center)
                    .spacing(10),
                ]
                .align_items(Alignment::Start)]
                .push_maybe(match self.shellcode_save_type() {
                    ShellcodeSaveType::Local => None,
                    ShellcodeSaveType::Remote => Some(
                        column![
                            text("Upload Final Shellcode Remote Plug-in"),
                            row![
                                text_input("", self.upload_final_shellcode_remote_plugin())
                                    .on_input(
                                        MakerMessage::UploadFinalShellcodeRemotePluginChanged
                                    ),
                                choose_button().on_press(MakerMessage::ChooseFileClicked(
                                    ChooseFileType::UploadFinalShellcodeRemote
                                ))
                            ]
                            .align_items(Alignment::Center)
                            .spacing(10)
                        ]
                        .align_items(Alignment::Start)
                    ),
                })
                .width(Length::FillPortion(1))
                .align_items(Alignment::Center)
            ]
            .align_items(Alignment::Center)
            .spacing(10),
            column![
                text("Description"),
                text_editor(self.desc())
                    .on_action(MakerMessage::DescAction)
                    .height(Length::Fill)
            ]
            .align_items(Alignment::Start),
            column![row![
                button("Generate").on_press(MakerMessage::GenerateClicked)
            ]]
            .align_items(Alignment::Center)
            .width(Length::Fill),
        ]
        .align_items(Alignment::Start)
        .padding(20)
        .spacing(10);

        let version = text(format!("PumpBin  v{}", self.pumpbin_version()))
            .color(self.theme().extended_palette().primary.base.color);

        let b1n = button(
            Svg::new(Handle::from_memory(include_bytes!(
                "../../assets/svg/house-heart-fill.svg"
            )))
            .width(30)
            .height(30)
            .style(style::svg::svg_primary_base),
        )
        .style(button::text)
        .on_press(MakerMessage::B1nClicked);
        let github = button(
            Svg::new(Handle::from_memory(include_bytes!(
                "../../assets/svg/github.svg"
            )))
            .width(30)
            .height(30)
            .style(style::svg::svg_primary_base),
        )
        .style(button::text)
        .on_press(MakerMessage::GithubClicked);

        let theme_list = pick_list(
            Theme::ALL,
            Some(self.selected_theme.clone()),
            MakerMessage::ThemeChanged,
        );

        let footer = column![
            horizontal_rule(0),
            row![
                column![version]
                    .width(Length::Fill)
                    .align_items(Alignment::Start),
                column![row![b1n, github].align_items(Alignment::Center)]
                    .width(Length::Shrink)
                    .align_items(Alignment::Center),
                column![theme_list]
                    .width(Length::Fill)
                    .align_items(Alignment::End)
            ]
            .padding([0, 20])
            .align_items(Alignment::Center)
        ]
        .align_items(Alignment::Center);

        column![maker, footer].align_items(Alignment::Center)
    }

    pub fn theme(&self) -> Theme {
        self.selected_theme()
    }
}
