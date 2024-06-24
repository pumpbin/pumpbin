use std::{fs, path::PathBuf, time::Duration, usize};

use dirs::{desktop_dir, home_dir};
use iced::{
    advanced::Application,
    executor,
    widget::{
        button, column, horizontal_rule, pick_list, radio, row, svg::Handle, text, text_editor,
        text_input, Svg,
    },
    Alignment, Length, Renderer, Task, Theme,
};
use pumpbin::{
    plugin::{Bins, Plugin},
    svg_style, ShellcodeSaveType, FONT,
};
use pumpbin::{
    plugin::{EncryptType, Platforms},
    Pumpbin,
};
use rfd::AsyncFileDialog;

#[derive(Debug)]
struct Maker {
    plugin_name: String,
    author: String,
    version: String,
    prefix: String,
    max_len: String,
    shellcode_save_type: ShellcodeSaveType,
    size_holder: String,
    encrypt_type: EncryptType,
    windows_exe: String,
    windows_lib: String,
    linux_exe: String,
    linux_lib: String,
    darwin_exe: String,
    darwin_lib: String,
    desc: text_editor::Content,
    message: String,
    selected_theme: Theme,
}

impl Default for Maker {
    fn default() -> Self {
        Self {
            plugin_name: Default::default(),
            author: Default::default(),
            version: Default::default(),
            prefix: Default::default(),
            max_len: Default::default(),
            shellcode_save_type: ShellcodeSaveType::Local,
            size_holder: Default::default(),
            encrypt_type: EncryptType::None,
            windows_exe: Default::default(),
            windows_lib: Default::default(),
            linux_exe: Default::default(),
            linux_lib: Default::default(),
            darwin_exe: Default::default(),
            darwin_lib: Default::default(),
            desc: text_editor::Content::new(),
            message: "Welcom to PumpBin Maker.".to_string(),
            selected_theme: Theme::CatppuccinMacchiato,
        }
    }
}

impl Maker {
    fn show_message(&mut self, message: String) -> Task<MakerMessage> {
        self.message = message;
        let wait = async {
            tokio::time::sleep(Duration::from_secs(3)).await;
        };
        Task::perform(wait, MakerMessage::ClearMessage)
    }

    fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    fn author(&self) -> &str {
        &self.author
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn prefix(&self) -> &str {
        &self.prefix
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

    fn encrypt_type(&self) -> &EncryptType {
        &self.encrypt_type
    }

    fn encrypt_type_mut(&mut self) -> &mut EncryptType {
        &mut self.encrypt_type
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

    fn desc(&self) -> &text_editor::Content {
        &self.desc
    }

    fn desc_mut(&mut self) -> &mut text_editor::Content {
        &mut self.desc
    }

    fn selected_theme(&self) -> Theme {
        self.selected_theme.clone()
    }
}

#[derive(Debug, Clone, Copy)]
enum ChooseFileType {
    WindowsExe,
    WindowsLib,
    LinuxExe,
    LinuxLib,
    DarwinExe,
    DarwinLib,
}

#[derive(Debug, Clone)]
enum MakerMessage {
    PluginNameChanged(String),
    AuthorChanged(String),
    VersionChanged(String),
    PrefixChanged(String),
    MaxLenChanged(String),
    ShellcodeSaveTypeChanged(ShellcodeSaveType),
    SizeHolderChanged(String),
    EncryptTypeChanged(EncryptType),
    XorPassChanged(String),
    AesKeyChanged(String),
    AesNonceChanged(String),
    WindowsExeChanged(String),
    WindowsLibChanged(String),
    LinuxExeChanged(String),
    LinuxLibChanged(String),
    DarwinExeChanged(String),
    DarwinLibChanged(String),
    DescAction(text_editor::Action),
    GenerateClicked,
    GenerateDone(Result<(), String>),
    ChooseFileClicked(ChooseFileType),
    WindowsExeChooseDone(Result<String, String>),
    WindowsLibChooseDone(Result<String, String>),
    LinuxExeChooseDone(Result<String, String>),
    LinuxLibChooseDone(Result<String, String>),
    DarwinExeChooseDone(Result<String, String>),
    DarwinLibChooseDone(Result<String, String>),
    B1nClicked,
    GithubClicked,
    ThemeChanged(Theme),
    ClearMessage(()),
}

impl Application for Maker {
    type Executor = executor::Default;
    type Flags = ();
    type Message = MakerMessage;
    type Theme = Theme;
    type Renderer = Renderer;

    fn new(_flags: Self::Flags) -> (Self, iced::Task<Self::Message>) {
        (Self::default(), Task::none())
    }

    fn title(&self) -> String {
        "PumpBin Maker".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            MakerMessage::PluginNameChanged(x) => {
                self.plugin_name = x;
                Task::none()
            }
            MakerMessage::AuthorChanged(x) => {
                self.author = x;
                Task::none()
            }
            MakerMessage::VersionChanged(x) => {
                self.version = x;
                Task::none()
            }
            MakerMessage::PrefixChanged(x) => {
                self.prefix = x;
                Task::none()
            }
            MakerMessage::MaxLenChanged(x) => {
                self.max_len = x;
                Task::none()
            }
            MakerMessage::ShellcodeSaveTypeChanged(x) => {
                self.shellcode_save_type = x;
                Task::none()
            }
            MakerMessage::SizeHolderChanged(x) => {
                self.size_holder = x;
                Task::none()
            }

            MakerMessage::EncryptTypeChanged(x) => {
                self.encrypt_type = x;
                Task::none()
            }
            MakerMessage::XorPassChanged(x) => {
                if let EncryptType::Xor(xor) = self.encrypt_type_mut() {
                    *xor = x.as_bytes().to_vec();
                }
                Task::none()
            }
            MakerMessage::AesKeyChanged(x) => {
                if let EncryptType::AesGcm(aes_gcm) = self.encrypt_type_mut() {
                    *aes_gcm.key_holder_mut() = x.as_bytes().to_vec();
                }
                Task::none()
            }
            MakerMessage::AesNonceChanged(x) => {
                if let EncryptType::AesGcm(aes_gcm) = self.encrypt_type_mut() {
                    *aes_gcm.nonce_holder_mut() = x.as_bytes().to_vec();
                }
                Task::none()
            }
            MakerMessage::WindowsExeChanged(x) => {
                self.windows_exe = x;
                Task::none()
            }
            MakerMessage::WindowsLibChanged(x) => {
                self.windows_lib = x;
                Task::none()
            }
            MakerMessage::LinuxExeChanged(x) => {
                self.linux_exe = x;
                Task::none()
            }
            MakerMessage::LinuxLibChanged(x) => {
                self.linux_lib = x;
                Task::none()
            }
            MakerMessage::DarwinExeChanged(x) => {
                self.darwin_exe = x;
                Task::none()
            }
            MakerMessage::DarwinLibChanged(x) => {
                self.darwin_lib = x;
                Task::none()
            }
            MakerMessage::DescAction(x) => {
                self.desc_mut().perform(x);
                Task::none()
            }
            MakerMessage::GenerateClicked => {
                if self.plugin_name().is_empty() {
                    return self.show_message("Plugin Name is empty.".to_string());
                }

                if self.prefix().is_empty() {
                    return self.show_message("Prefix is empty.".to_string());
                }

                if let ShellcodeSaveType::Local = self.shellcode_save_type() {
                    if self.size_holder().is_empty() {
                        return self.show_message("Size Holder is empty.".to_string());
                    }
                }

                let max_len;

                if let Ok(max) = self.max_len().parse::<usize>() {
                    max_len = max;
                } else {
                    return self.show_message("MaxLen numeric only.".to_string());
                }

                match self.encrypt_type() {
                    EncryptType::None => (),
                    EncryptType::Xor(x) => {
                        if x.is_empty() {
                            return self.show_message("Xor Pass is empty.".to_string());
                        }
                    }
                    EncryptType::AesGcm(x) => {
                        if x.key_holder().is_empty() {
                            return self.show_message("AesGcm Key is empty.".to_string());
                        } else if x.nonce_holder().is_empty() {
                            return self.show_message("AesGcm Nonce is empty.".to_string());
                        }
                    }
                }

                let windows_exe_path = PathBuf::from(self.windows_exe());
                let windows_dll_path = PathBuf::from(self.windows_lib());
                let windows = match (
                    windows_exe_path.exists() && windows_exe_path.is_file(),
                    windows_dll_path.exists() && windows_dll_path.is_file(),
                ) {
                    (false, false) => None,
                    _ => Some(Bins {
                        executable: if let Ok(bin) = fs::read(&windows_exe_path) {
                            Some(bin)
                        } else {
                            None
                        },
                        dynamic_library: if let Ok(bin) = fs::read(&windows_dll_path) {
                            Some(bin)
                        } else {
                            None
                        },
                    }),
                };

                let linux_exe_path = PathBuf::from(self.linux_exe());
                let linux_dll_path = PathBuf::from(self.linux_lib());
                let linux = match (
                    linux_exe_path.exists() && linux_exe_path.is_file(),
                    linux_dll_path.exists() && linux_dll_path.is_file(),
                ) {
                    (false, false) => None,
                    _ => Some(Bins {
                        executable: if let Ok(bin) = fs::read(&linux_exe_path) {
                            Some(bin)
                        } else {
                            None
                        },
                        dynamic_library: if let Ok(bin) = fs::read(&linux_dll_path) {
                            Some(bin)
                        } else {
                            None
                        },
                    }),
                };

                let darwin_exe_path = PathBuf::from(self.darwin_exe());
                let darwin_dll_path = PathBuf::from(self.darwin_lib());
                let darwin = match (
                    darwin_exe_path.exists() && darwin_exe_path.is_file(),
                    darwin_dll_path.exists() && darwin_dll_path.is_file(),
                ) {
                    (false, false) => None,
                    _ => Some(Bins {
                        executable: if let Ok(bin) = fs::read(&darwin_exe_path) {
                            Some(bin)
                        } else {
                            None
                        },
                        dynamic_library: if let Ok(bin) = fs::read(&darwin_dll_path) {
                            Some(bin)
                        } else {
                            None
                        },
                    }),
                };

                let plugin = Plugin {
                    plugin_name: self.plugin_name().to_string(),
                    author: match self.author().is_empty() {
                        true => None,
                        false => Some(self.author().to_string()),
                    },
                    version: match self.version().is_empty() {
                        true => None,
                        false => Some(self.version().to_string()),
                    },
                    desc: match self.desc().text().is_empty() {
                        true => None,
                        false => Some(self.desc().text()),
                    },
                    prefix: self.prefix().as_bytes().to_vec(),
                    size_holder: match self.shellcode_save_type() {
                        ShellcodeSaveType::Local => Some(self.size_holder().as_bytes().to_vec()),
                        ShellcodeSaveType::Remote => None,
                    },
                    max_len,
                    encrypt_type: self.encrypt_type().to_owned(),
                    platforms: Platforms {
                        windows,
                        linux,
                        darwin,
                    },
                };

                let plugin_name = self.plugin_name().to_owned();
                let make_plugin = async move {
                    let file = AsyncFileDialog::new()
                        .set_directory(desktop_dir().unwrap_or(".".into()))
                        .set_file_name(format!("{}.b1n", plugin_name))
                        .set_can_create_directories(true)
                        .set_title("save plugin")
                        .save_file()
                        .await
                        .ok_or("Canceled plugin saving.".to_string())?;

                    plugin
                        .write_plugin(file.path())
                        .map_err(|_| "Write plugin failed.".to_string())?;

                    Ok(())
                };

                Task::perform(make_plugin, MakerMessage::GenerateDone)
            }
            MakerMessage::GenerateDone(x) => self.show_message(match x {
                Ok(_) => "Generate done.".to_string(),
                Err(e) => e,
            }),
            MakerMessage::ChooseFileClicked(x) => {
                let choose_file = async move {
                    AsyncFileDialog::new()
                        .set_directory(home_dir().unwrap_or(".".into()))
                        .set_title("choose file")
                        .pick_file()
                        .await
                        .map(|x| x.path().to_string_lossy().to_string())
                        .ok_or("Canceled file selection.".to_string())
                };

                Task::perform(
                    choose_file,
                    match x {
                        ChooseFileType::WindowsExe => MakerMessage::WindowsExeChooseDone,
                        ChooseFileType::WindowsLib => MakerMessage::WindowsLibChooseDone,
                        ChooseFileType::LinuxExe => MakerMessage::LinuxExeChooseDone,
                        ChooseFileType::LinuxLib => MakerMessage::LinuxLibChooseDone,
                        ChooseFileType::DarwinExe => MakerMessage::DarwinExeChooseDone,
                        ChooseFileType::DarwinLib => MakerMessage::DarwinLibChooseDone,
                    },
                )
            }
            MakerMessage::WindowsExeChooseDone(x) => {
                match x {
                    Ok(x) => self.windows_exe = x,
                    Err(x) => return self.show_message(x),
                }
                Task::none()
            }
            MakerMessage::WindowsLibChooseDone(x) => {
                match x {
                    Ok(x) => self.windows_lib = x,
                    Err(x) => return self.show_message(x),
                }
                Task::none()
            }
            MakerMessage::LinuxExeChooseDone(x) => {
                match x {
                    Ok(x) => self.linux_exe = x,
                    Err(x) => return self.show_message(x),
                }
                Task::none()
            }
            MakerMessage::LinuxLibChooseDone(x) => {
                match x {
                    Ok(x) => self.linux_lib = x,
                    Err(x) => return self.show_message(x),
                }
                Task::none()
            }
            MakerMessage::DarwinExeChooseDone(x) => {
                match x {
                    Ok(x) => self.darwin_exe = x,
                    Err(x) => return self.show_message(x),
                }
                Task::none()
            }
            MakerMessage::DarwinLibChooseDone(x) => {
                match x {
                    Ok(x) => self.darwin_lib = x,
                    Err(x) => return self.show_message(x),
                }
                Task::none()
            }
            MakerMessage::B1nClicked => {
                if open::that(env!("CARGO_PKG_HOMEPAGE")).is_err() {
                    return self.show_message("Open home failed.".into());
                }
                Task::none()
            }
            MakerMessage::GithubClicked => {
                if open::that(env!("CARGO_PKG_REPOSITORY")).is_err() {
                    return self.show_message("Open repo failed.".into());
                }
                Task::none()
            }
            MakerMessage::ThemeChanged(x) => {
                self.selected_theme = x;
                Task::none()
            }
            MakerMessage::ClearMessage(_) => {
                self.message = "Welcom to PumpBin Maker.".to_string();
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        let choose_button = || {
            button(
                Svg::new(Handle::from_memory(include_bytes!(
                    "../../assets/svg/three-dots.svg"
                )))
                .width(20),
            )
        };

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
                    text_input("", self.prefix()).on_input(MakerMessage::PrefixChanged),
                ]
                .align_items(Alignment::Start),
                column![
                    text("MaxLen"),
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
                text("Encrypt Type"),
                row![pick_list(
                    EncryptType::all(),
                    Some(self.encrypt_type()),
                    MakerMessage::EncryptTypeChanged
                )
                .handle(pick_list_handle())]
                .push_maybe(match self.encrypt_type() {
                    EncryptType::None => None,
                    EncryptType::Xor(x) => Some(
                        row![
                            text("Pass:"),
                            text_input("", &String::from_utf8_lossy(x))
                                .on_input(MakerMessage::XorPassChanged)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center)
                    ),
                    EncryptType::AesGcm(x) => Some(
                        row![
                            text("Key:"),
                            text_input("", &String::from_utf8_lossy(x.key_holder()))
                                .on_input(MakerMessage::AesKeyChanged),
                            text("Nonce:"),
                            text_input("", &String::from_utf8_lossy(x.nonce_holder()))
                                .on_input(MakerMessage::AesNonceChanged)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center)
                    ),
                })
                .spacing(20)
                .align_items(Alignment::Center)
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

        let message = row![text(" ").size(25), text(&self.message)].align_items(Alignment::Center);
        let b1n = button(
            Svg::new(Handle::from_memory(include_bytes!(
                "../../assets/svg/house-heart-fill.svg"
            )))
            .width(30)
            .height(30)
            .style(svg_style::svg_primary_base),
        )
        .style(button::text)
        .on_press(MakerMessage::B1nClicked);
        let github = button(
            Svg::new(Handle::from_memory(include_bytes!(
                "../../assets/svg/github.svg"
            )))
            .width(30)
            .height(30)
            .style(svg_style::svg_primary_base),
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
            .padding([0, 20])
            .align_items(Alignment::Center)
        ]
        .align_items(Alignment::Center);

        column![maker, footer].align_items(Alignment::Center).into()
    }

    fn theme(&self) -> Self::Theme {
        self.selected_theme()
    }
}

fn main() -> iced::Result {
    Maker::run(Pumpbin::settings())
}
