use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug)]
pub struct AddArgs {
    pub target_file: PathBuf,
    pub source_dir: PathBuf,
    pub verbose: bool,
}

pub fn add(args: AddArgs) -> Result<()> {
    println!(
        "Adding new file '{}' to source directory: {}",
        args.target_file.display(),
        args.source_dir.display(),
    );
    todo!("make target path relative to source directory");
    std::fs::copy(&args.target_file, &args.source_dir).context("copy file to source directory")?;
    Ok(())
}
