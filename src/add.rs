use crate::files::{copy_file_mode, get_relative_path};
use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug)]
pub struct AddArgs {
    pub target_file: PathBuf,
    pub target_base: PathBuf,
    pub source_dir: PathBuf,
    pub verbose: bool,
}

pub fn add(args: AddArgs) -> Result<()> {
    let target_file = if args.target_file.as_path().is_absolute() {
        args.target_file
    } else {
        std::env::current_dir()
            .context("get current working directory from environment")?
            .join(&args.target_file)
    };
    let relative_target_file =
        get_relative_path(&args.target_base, &target_file).context("get relative target file")?;
    let target_path = args.source_dir.join(relative_target_file);
    println!(
        "Adding new file to source directory: {}",
        target_path.display(),
    );
    std::fs::copy(&target_file, &target_path).context("copy file to source directory")?;
    copy_file_mode(&target_file, &target_path)?;
    Ok(())
}
