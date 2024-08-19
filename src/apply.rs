use crate::{config::Config, files};
use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use std::path::{Path, PathBuf};

const STAGING_DIR_TEMPLATE: &str = "homux.staging.XXXXXXXXX";

#[derive(Debug, Clone, Parser)]
pub struct Args {
    /// Dry run (apply to temporary directory instead of home directory)
    #[arg(short = 'd', long)]
    dry_run: bool,
    /// Print more verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
}

pub fn apply(args: &Args, config: &Config) -> Result<()> {
    let target_dir = if args.dry_run {
        config.dirs.dry_run.clone()
    } else {
        config.dirs.home.clone()
    };

    let staging_dir = get_staging_dir()?;
    println!(
        "{} at directory: {}",
        "Staging".green().bold(),
        staging_dir.display()
    );
    stage_source(config, &staging_dir, args.verbose).context("stage source")?;
    println!(
        "{} to target directory: {}",
        "Applying".green().bold(),
        target_dir.display()
    );
    let apply_result = files::copy_directory_full(&staging_dir, &target_dir)
        .context("copy staging recursively to target");
    println!(
        "{} up staging directory: {}",
        "Cleaning".green().bold(),
        staging_dir.display()
    );
    cleanup_staging(&staging_dir)?;
    apply_result?;
    println!("{}", "Done.".green().bold());
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

fn stage_source(config: &Config, staging_dir: &Path, verbose: bool) -> Result<()> {
    let source_dir_contents = files::walk_dir(&config.dirs.source)?;
    for dir_path in source_dir_contents.dirs {
        let relative_path = files::get_relative_path(&config.dirs.source, &dir_path)
            .with_context(|| format!("non-relative path: {}", dir_path.display()))?;
        let staging_path = staging_dir.join(relative_path);
        std::fs::create_dir_all(&staging_path).context("failed to create staging subdirectory")?;
    }
    for file_path in source_dir_contents.files {
        let filesize = std::fs::metadata(&file_path)
            .with_context(|| format!("failed to read file metadata: {}", file_path.display()))?
            .len();
        let relative_path = files::get_relative_path(&config.dirs.source, &file_path)
            .with_context(|| format!("non-relative path: {}", file_path.display()))?;
        if verbose {
            print!(
                "{} {}",
                "Processing".green().dimmed(),
                relative_path.display()
            );
        }
        let staging_path = staging_dir.join(relative_path);
        // Determine if file is to be matchpicked
        let matchpickable_text = if filesize <= config.matchpick.max_file_size {
            let bytes = std::fs::read(&file_path)
                .with_context(|| format!("failed to read file {}", file_path.display()))?;
            match String::from_utf8(bytes) {
                Ok(text) => {
                    if verbose {
                        println!("{}", " [matchpicking]".yellow().dimmed());
                    };
                    Some(text)
                }
                Err(utf8error) => {
                    if verbose {
                        print!("{}", format!(" [{utf8error}]").yellow().bold());
                    };
                    None
                }
            }
        } else {
            if verbose {
                println!();
            }
            None
        };
        // Matchpick / copy
        if let Some(original_text) = matchpickable_text {
            let fixed_text = matchpick::process(
                &original_text,
                Some(config.hostname.clone()),
                &config.matchpick.enter_pattern,
                &config.matchpick.exit_pattern,
            )?;
            std::fs::write(&staging_path, fixed_text).context("failed to write to staging dir")?;
        } else {
            std::fs::copy(&file_path, &staging_path).context("failed to copy to staging dir")?;
        }
        files::copy_file_mode(&file_path, &staging_path)?;
    }
    Ok(())
}
