use crate::files::{copy_file_mode, get_relative_path};
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    /// New file
    #[arg()]
    new_file: PathBuf,
}

pub fn add(args: &Args, config: &crate::config::Config) -> Result<()> {
    let absolute_new_file = if args.new_file.as_path().is_absolute() {
        args.new_file.clone()
    } else {
        std::env::current_dir()
            .context("get current working directory from environment")?
            .join(&args.new_file)
    };
    let relative_new_file = get_relative_path(&config.dirs.home, &absolute_new_file)
        .context("get relative target file")?;
    let target_path = config.dirs.source.join(relative_new_file);
    println!(
        "Adding new file to source directory: {}",
        target_path.display(),
    );
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).context("create sub-directory in source directory")?;
    }
    std::fs::copy(&absolute_new_file, &target_path).context("copy file to source directory")?;
    copy_file_mode(&absolute_new_file, &target_path)?;
    Ok(())
}
