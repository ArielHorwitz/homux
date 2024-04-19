use crate::files::{copy_directory_full, get_relative_path, walk_dir, copy_file_mode};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const STAGING_DIR_TEMPLATE: &str = "homux.staging.XXXXXXXXX";
const ENTER_PATTERN: &str = "~>>>";
const EXIT_PATTERN: &str = "~<<<";

#[derive(Debug)]
pub struct ApplyArgs {
    pub source_dir: PathBuf,
    pub target_dir: PathBuf,
    pub hostname: String,
    pub max_file_size: u64,
    pub verbose: bool,
}

pub fn apply(args: ApplyArgs) -> Result<()> {
    let staging_dir = get_staging_dir()?;
    println!("Staging at directory: {}", staging_dir.display());
    stage_source(&args, &staging_dir).context("stage source")?;
    println!(
        "Applying to target directory: {}",
        args.target_dir.display()
    );
    let apply_result = copy_directory_full(&staging_dir, &args.target_dir)
        .context("copy staging recursively to target");
    println!("Cleaning up staging directory: {}", staging_dir.display());
    cleanup_staging(&staging_dir)?;
    apply_result?;
    println!("Done.");
    Ok(())
}

fn get_staging_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(
        String::from_utf8(
            std::process::Command::new("mktemp")
                .arg("--tmpdir")
                .arg("-d")
                .arg(STAGING_DIR_TEMPLATE)
                .output()
                .context("failed to get temporary directory")?
                .stdout,
        )?
        .trim(),
    ))
}

fn cleanup_staging(staging_dir: &Path) -> Result<()> {
    std::fs::remove_dir_all(staging_dir).with_context(|| {
        format!(
            "failed to remove staging directory {}",
            staging_dir.display()
        )
    })?;
    Ok(())
}

fn stage_source(args: &ApplyArgs, staging_dir: &Path) -> Result<()> {
    let source_dir_contents = walk_dir(&args.source_dir)?;
    if args.verbose {
        println!("{source_dir_contents:#?}");
    }
    for dir_path in source_dir_contents.dirs {
        let relative_path = get_relative_path(&args.source_dir, &dir_path)
            .with_context(|| format!("non-relative path: {}", dir_path.display()))?;
        let staging_path = staging_dir.join(relative_path);
        std::fs::create_dir_all(&staging_path).context("failed to create staging subdirectory")?;
    }
    for file_path in source_dir_contents.files {
        let filesize = std::fs::metadata(&file_path)
            .with_context(|| format!("failed to read file metadata: {}", file_path.display()))?
            .len();
        let relative_path = get_relative_path(&args.source_dir, &file_path)
            .with_context(|| format!("non-relative path: {}", file_path.display()))?;
        if args.verbose {
            println!("Processing {}", relative_path.display());
        }
        let staging_path = staging_dir.join(relative_path);
        if filesize <= args.max_file_size {
            let original_text = std::fs::read_to_string(&file_path)
                .with_context(|| format!("failed to read file {}", file_path.display()))?;
            let fixed_text = matchpick::process(
                &original_text,
                Some(args.hostname.clone()),
                ENTER_PATTERN,
                EXIT_PATTERN,
            )?;
            std::fs::write(&staging_path, fixed_text).context("failed to write to staging dir")?;
        } else {
            std::fs::copy(&file_path, &staging_path).context("failed to copy to staging dir")?;
        }
        copy_file_mode(&file_path, &staging_path)?;
    }
    Ok(())
}
