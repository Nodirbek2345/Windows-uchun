use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub proxy: ProxyConfig,
    pub target_sites: Vec<String>,
    pub text_filter: TextFilterConfig,
    pub image_filter: ImageFilterConfig,
    pub mapping: MappingConfig,
    pub logging: LoggingConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyConfig {
    pub port: u16,
    pub host: String,
    pub ssl_insecure: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextFilterConfig {
    pub enabled: bool,
    pub methods: MethodsConfig,
    pub detect: DetectConfig,
    pub mask_style: String,
    pub blacklist: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MethodsConfig {
    pub regex: bool,
    pub blacklist: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DetectConfig {
    pub names: bool,
    pub phone: bool,
    pub email: bool,
    pub card_number: bool,
    pub address: bool,
    pub organization: bool,
    pub passport: bool,
    pub pinfl: bool,
    pub date_of_birth: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageFilterConfig {
    pub enabled: bool,
    pub face: FaceConfig,
    pub ocr: OcrConfig,
    pub metadata: MetadataConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FaceConfig {
    pub enabled: bool,
    pub method: String,
    pub blur_radius: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OcrConfig {
    pub enabled: bool,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetadataConfig {
    pub strip_gps: bool,
    pub strip_device: bool,
    pub strip_datetime: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MappingConfig {
    pub storage: String,
    pub ttl_minutes: u64,
    pub encrypt: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub level: String,
    pub max_entries: usize,
    pub save_to_file: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    pub language: String,
    pub theme: String,
    pub start_minimized: bool,
    pub autostart: bool,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
