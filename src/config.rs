use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf};
use toml;

const DEFAULT_CONFIG_DIR: &str = ".config/homux";
const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_TOML_TEMPLATE: &str = include_str!("../config_template.toml");

#[derive(Debug, Deserialize)]
pub struct UserConfiguration {
    pub source: PathBuf,
}

#[derive(Debug)]
pub struct Config {
    pub source: PathBuf,
}

impl TryFrom<&UserConfiguration> for Config {
    type Error = anyhow::Error;

    fn try_from(user_config: &UserConfiguration) -> Result<Self> {
        let source = crate::files::get_home_dir()?.join(&user_config.source);
        Ok(Self { source })
    }
}

pub fn parse_configuration_file(file_path: Option<PathBuf>) -> Result<Config> {
    let config_file_path = if let Some(file_path) = file_path {
        file_path
    } else {
        get_default_config_path()?
    };
    let user_config_string =
        std::fs::read_to_string(config_file_path).context("read user config file")?;
    let config: UserConfiguration =
        toml::from_str(&user_config_string).context("parse user config file")?;
    Config::try_from(&config)
}

pub fn generate_default_configuration_file() -> Result<()> {
    fs::create_dir_all(get_default_config_dir()?).context("create configuration directory")?;
    let config_file_path = get_default_config_path()?;
    if !config_file_path.is_file() {
        fs::write(config_file_path, CONFIG_TOML_TEMPLATE)
            .context("write configuration template")?;
    }
    Ok(())
}

fn get_default_config_dir() -> Result<PathBuf> {
    Ok(crate::files::get_home_dir()?.join(DEFAULT_CONFIG_DIR))
}

pub fn get_default_config_path() -> Result<PathBuf> {
    Ok(get_default_config_dir()?.join(DEFAULT_CONFIG_FILE_NAME))
}
