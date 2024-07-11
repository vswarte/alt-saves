use std::fs;
use std::path;
use std::sync;
use serde::Deserialize;

const CONFIG_FILE: &str = "./altsaves.toml";

static mut CONFIG: sync::OnceLock<Config> = sync::OnceLock::new();

#[derive(Deserialize)]
pub struct Config {
    pub extension: String,
    pub seamless_extension: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            extension: ".mod".to_string(),
            seamless_extension: Some(".mod.co2".to_string())
        }
    }
}

fn get_config_file() -> &'static Config {
    unsafe {
        CONFIG.get_or_init(|| read_config_file().unwrap_or_default())
    }
}

fn read_config_file() -> Option<Config> {
    path::absolute(path::PathBuf::from(CONFIG_FILE))
        .map(|p| fs::read_to_string(p).ok()).ok()
        .flatten()
        .and_then(|f| toml::from_str(f.as_str()).ok())
}

pub fn get_rewrite_extension() -> &'static str {
    get_config_file().extension.as_ref()
}

pub fn get_seamless_rewrite_extension() -> Option<&'static String> {
    get_config_file().seamless_extension.as_ref()
}
