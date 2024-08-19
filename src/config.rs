use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf};
use toml;

const CONFIG_DIR: &str = ".config/homux";
const CONFIG_FILE: &str = "config.toml";
const CONFIG_TOML_TEMPLATE: &str = include_str!("../config_template.toml");

#[derive(Debug, Deserialize)]
struct UserConfiguration {
    pub hostname: Option<String>,
    pub directories: UserDirectories,
    pub matchpick: Matchpick,
}

#[derive(Debug, Deserialize)]
pub struct UserDirectories {
    pub source: PathBuf,
    pub dry_run: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Directories {
    pub home: PathBuf,
    pub source: PathBuf,
    pub dry_run: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Matchpick {
    pub max_file_size: u64,
    pub enter_pattern: String,
    pub exit_pattern: String,
}

#[derive(Debug)]
pub struct Config {
    pub hostname: String,
    pub dirs: Directories,
    pub matchpick: Matchpick,
}

impl Config {
    pub fn new(generate_missing: bool) -> Result<Config> {
        let home_dir = crate::files::get_home_dir().context("get home dir")?;
        let config_dir = home_dir.join(CONFIG_DIR);
        let config_file = config_dir.join(CONFIG_FILE);

        if !config_file.exists() && generate_missing {
            if let Some(config_dir) = config_file.parent() {
                fs::create_dir_all(config_dir).context("create configuration directory")?;
            }
            fs::write(&config_file, CONFIG_TOML_TEMPLATE)
                .context("write configuration template")?;
        }

        let user_config: UserConfiguration = toml::from_str(
            &std::fs::read_to_string(&config_file).context("read user config file")?,
        )
        .context("parse user config file")?;

        let hostname = if let Some(hostname) = user_config.hostname {
            hostname
        } else {
            get_machine_hostname().context("get machine hostname")?
        };

        let dirs = Directories {
            source: home_dir.join(&user_config.directories.source),
            dry_run: home_dir.join(&user_config.directories.dry_run),
            home: home_dir,
        };

        Ok(Config {
            hostname,
            dirs,
            matchpick: user_config.matchpick,
        })
    }
}

fn get_machine_hostname() -> Result<String> {
    let stdout = std::process::Command::new("hostnamectl")
        .arg("hostname")
        .output()
        .context("failed to get hostname")?
        .stdout;
    let hostname = String::from_utf8(stdout).context("parse machine hostname as utf8")?;
    Ok(hostname.trim().to_owned())
}
