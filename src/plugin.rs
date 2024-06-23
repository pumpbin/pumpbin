use std::{collections::HashMap, fmt::Display, fs, ops::Not, path::Path};

use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use anyhow::anyhow;
use bincode::{decode_from_slice, encode_to_vec, Decode, Encode};
use dirs::data_dir;

// 500 MiB
const LIMIT: usize = 1024 * 1024 * 500;
pub const BINCODE_PLUGIN_CONFIG: bincode::config::Configuration<
    bincode::config::LittleEndian,
    bincode::config::Varint,
    bincode::config::Limit<LIMIT>,
> = bincode::config::standard().with_limit();
const BINCODE_PLUGINS_CONFIG: bincode::config::Configuration = bincode::config::standard();

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct AesGcmPass {
    key_holder: Vec<u8>,
    nonce_holder: Vec<u8>,
}

impl AesGcmPass {
    pub fn key_holder(&self) -> &[u8] {
        &self.key_holder
    }

    pub fn key_holder_mut(&mut self) -> &mut Vec<u8> {
        &mut self.key_holder
    }

    pub fn nonce_holder(&self) -> &[u8] {
        &self.nonce_holder
    }

    pub fn nonce_holder_mut(&mut self) -> &mut Vec<u8> {
        &mut self.nonce_holder
    }
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum EncryptType {
    None,
    Xor(Vec<u8>),
    AesGcm(AesGcmPass),
}

impl Display for EncryptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptType::None => write!(f, "None"),
            EncryptType::Xor(_) => write!(f, "Xor"),
            EncryptType::AesGcm(_) => write!(f, "AesGcm"),
        }
    }
}

impl EncryptType {
    pub const fn all() -> [EncryptType; 3] {
        [
            EncryptType::None,
            EncryptType::Xor(vec![]),
            EncryptType::AesGcm(AesGcmPass {
                key_holder: vec![],
                nonce_holder: vec![],
            }),
        ]
    }
    pub fn encrypt(&self, path: &Path) -> anyhow::Result<Vec<u8>> {
        let data = fs::read(path)?;

        match self {
            EncryptType::None => Ok(data),
            EncryptType::Xor(x) => Ok(data
                .iter()
                .enumerate()
                .map(|(i, byte)| byte ^ x[i % x.len()])
                .collect()),
            EncryptType::AesGcm(x) => {
                let key = Key::<Aes256Gcm>::from_slice(x.key_holder());
                let aes = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(x.nonce_holder());
                aes.encrypt(nonce, data.as_slice()).map_err(|e| anyhow!(e))
            }
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Bins {
    pub executable: Option<Vec<u8>>,
    pub dynamic_library: Option<Vec<u8>>,
}

impl Bins {
    pub fn executable(&self) -> Option<&Vec<u8>> {
        self.executable.as_ref()
    }

    pub fn dynamic_library(&self) -> Option<&Vec<u8>> {
        self.dynamic_library.as_ref()
    }
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Platforms {
    pub windows: Option<Bins>,
    pub linux: Option<Bins>,
    pub darwin: Option<Bins>,
}

impl Platforms {
    pub fn windows(&self) -> Option<&Bins> {
        self.windows.as_ref()
    }

    pub fn linux(&self) -> Option<&Bins> {
        self.linux.as_ref()
    }

    pub fn darwin(&self) -> Option<&Bins> {
        self.darwin.as_ref()
    }
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Plugin {
    pub plugin_name: String,
    pub author: Option<String>,
    pub version: Option<String>,
    pub desc: Option<String>,
    pub prefix: Vec<u8>,
    pub size_holder: Option<Vec<u8>>,
    pub max_len: usize,
    pub encrypt_type: EncryptType,
    pub platforms: Platforms,
}

impl Plugin {
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    pub fn author(&self) -> Option<&String> {
        self.author.as_ref()
    }

    pub fn version(&self) -> Option<&String> {
        self.version.as_ref()
    }

    pub fn desc(&self) -> Option<&String> {
        self.desc.as_ref()
    }

    pub fn prefix(&self) -> &[u8] {
        &self.prefix
    }

    pub fn size_holder(&self) -> Option<&Vec<u8>> {
        self.size_holder.as_ref()
    }

    pub fn max_len(&self) -> usize {
        self.max_len
    }

    pub fn encrypt_type(&self) -> &EncryptType {
        &self.encrypt_type
    }

    pub fn platforms(&self) -> &Platforms {
        &self.platforms
    }
}

impl Plugin {
    pub fn reade_plugin(path: &Path) -> anyhow::Result<Plugin> {
        let buf = fs::read(path)?;
        let (plugin, _) = decode_from_slice(buf.as_slice(), BINCODE_PLUGIN_CONFIG)?;
        Ok(plugin)
    }

    pub fn write_plugin(&self, path: &Path) -> anyhow::Result<()> {
        let buf = encode_to_vec(self, BINCODE_PLUGIN_CONFIG)?;
        fs::write(path, buf.as_slice())?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Encode, Decode, PartialEq, Eq)]
pub struct Plugins(pub HashMap<String, Plugin>);

impl Plugins {
    pub fn reade_plugins() -> anyhow::Result<Plugins> {
        let mut plugins_path = data_dir().ok_or(anyhow::anyhow!("data_dir is none."))?;
        plugins_path.push("pumpbin");
        plugins_path.push("plugins");

        if plugins_path.exists() && plugins_path.is_file() {
            let buf = fs::read(plugins_path)?;
            let (plugins, _) = decode_from_slice(buf.as_slice(), BINCODE_PLUGINS_CONFIG)?;
            Ok(plugins)
        } else {
            anyhow::bail!("file not exists.")
        }
    }

    pub fn uptade_plugins(&self) -> anyhow::Result<()> {
        let buf = encode_to_vec(self, BINCODE_PLUGINS_CONFIG)?;

        let mut plugins_path = data_dir().ok_or(anyhow::anyhow!("data_dir is none."))?;
        plugins_path.push("pumpbin");
        if plugins_path.exists().not() {
            fs::create_dir_all(&plugins_path)?;
        } else if plugins_path.exists() && plugins_path.is_dir().not() {
            fs::remove_file(&plugins_path)?;
            fs::create_dir_all(&plugins_path)?;
        }
        plugins_path.push("plugins");
        fs::write(plugins_path, buf)?;

        Ok(())
    }
}
