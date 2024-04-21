use anyhow::{Context, Result};
use std::path::PathBuf;

pub mod add;
pub mod apply;
pub mod files;


pub fn get_default_source_directory() -> Result<PathBuf> {
    Ok(files::get_home_dir()?.join(".local/share/homux/source"))
}

pub fn get_machine_hostname() -> Result<String> {
    Ok(String::from_utf8(
        std::process::Command::new("hostnamectl")
            .arg("hostname")
            .output()
            .context("failed to get hostname")?
            .stdout,
    )?
    .trim()
    .to_owned())
}
