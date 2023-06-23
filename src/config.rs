use std::fs;
use std::path;
use std::sync;
use serde::Deserialize;

const CONFIG_FILE: &str = "./altsaves.toml";

static mut CONFIG: sync::OnceLock<Config> = sync::OnceLock::new();

#[derive(Clone, Deserialize)]
pub struct Config {
    pub extension: String,
    pub seamless_extension: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config { extension: ".mod".to_string(), seamless_extension: Some(".mod.co2".to_string()) }
    }
}

fn get_config_file() -> Config {
    unsafe {
        CONFIG.get_or_init(|| read_config_file().unwrap_or_else(|| Config::default())).clone()
    }
}

fn read_config_file() -> Option<Config> {
    path::absolute(path::PathBuf::from(CONFIG_FILE))
        .map(|p| fs::read_to_string(p).ok()).ok()
        .flatten()
        .map(|f| toml::from_str(f.as_str()).ok())
        .flatten()
}

pub fn get_rewrite_extension() -> String {
    get_config_file().extension.clone()
}

pub fn get_seamless_rewrite_extension() -> Option<String> {
    get_config_file().seamless_extension.clone()
}