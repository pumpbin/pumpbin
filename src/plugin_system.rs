use std::time::Duration;

use extism::{Manifest, Wasm};
use serde::{Deserialize, Serialize};

pub fn run_plugin<T: Serialize>(wasm: &[u8], func: &str, input: &T) -> anyhow::Result<Vec<u8>> {
    let data = Wasm::data(wasm.to_vec());
    let manifest = Manifest::new([data])
        .with_timeout(Duration::from_secs(5))
        .with_allowed_host("*");
    let mut plugin = extism::Plugin::new(manifest, [], true)?;

    plugin.call::<Vec<u8>, Vec<u8>>(func, serde_json::to_vec(input)?)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EncryptShellcodeInput {
    pub shellcode: Vec<u8>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Pass {
    pub holder: Vec<u8>,
    pub replace_by: Vec<u8>,
}

impl Pass {
    pub fn holder(&self) -> &[u8] {
        &self.holder
    }

    pub fn replace_by(&self) -> &[u8] {
        &self.replace_by
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EncryptShellcodeOutput {
    pub encrypted: Vec<u8>,
    pub pass: Vec<Pass>,
}

impl EncryptShellcodeOutput {
    pub fn encrypted(&self) -> &[u8] {
        &self.encrypted
    }

    pub fn pass(&self) -> &[Pass] {
        &self.pass
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FormatEncryptedShellcodeInput {
    pub shellcode: Vec<u8>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FormatEncryptedShellcodeOutput {
    pub formated_shellcode: Vec<u8>,
}

impl FormatEncryptedShellcodeOutput {
    pub fn formated_shellcode(&self) -> &[u8] {
        &self.formated_shellcode
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FormatUrlRemoteInput {
    pub url: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FormatUrlRemoteOutput {
    pub formated_url: String,
}

impl FormatUrlRemoteOutput {
    pub fn formated_url(&self) -> &str {
        &self.formated_url
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UploadFinalShellcodeRemoteInput {
    pub final_shellcode: Vec<u8>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UploadFinalShellcodeRemoteOutput {
    pub url: String,
}

impl UploadFinalShellcodeRemoteOutput {
    pub fn url(&self) -> &str {
        &self.url
    }
}
