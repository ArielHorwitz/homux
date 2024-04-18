use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(name = "homux")]
#[clap(about = "Synchronize your home directory across host machines.")]
#[clap(author = "https://ariel.ninja")]
#[clap(version)]
struct Args {
    /// Operation
    #[command(subcommand)]
    operation: Operation,
}

#[derive(Debug, Parser)]
enum Operation {
    /// Apply a source directory
    Apply(ApplyArgs),
    /// Add a new file to the source directory
    Add(AddArgs),
}

#[derive(Debug, Parser)]
struct ApplyArgs {
    /// Source directory
    #[arg(short = 's', long)]
    source_dir: Option<PathBuf>,
    /// Use a custom hostname
    #[arg(short = 'n', long)]
    hostname: Option<String>,
    /// Dry run (apply to temporary directory instead of home directory)
    #[arg(short = 'd', long)]
    dry_run: bool,
    /// Show more verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
}

#[derive(Debug, Parser)]
struct AddArgs {
    /// New file
    #[arg()]
    new_file: PathBuf,
    /// Source directory
    #[arg(short = 's', long)]
    source_dir: Option<PathBuf>,
    /// Show more verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.operation {
        Operation::Apply(args) => apply(args)?,
        Operation::Add(args) => add(args)?,
    }
    Ok(())
}

fn apply(args: ApplyArgs) -> Result<()> {
    let source_dir = if let Some(source_dir) = args.source_dir {
        // TODO: check if suspect that source directory is not a home directory
        source_dir
    } else {
        get_default_source_directory().context("get default source directory")?
    };
    let target_dir = if args.dry_run {
        PathBuf::from(String::from("/tmp/homux.dry_run"))
    } else {
        unimplemented!("only dry run is implemented")
    };
    let hostname = if let Some(hostname) = args.hostname {
        hostname
    } else {
        get_machine_hostname().context("get hostname")?
    };
    let apply_args = homux::apply::ApplyArgs {
        source_dir,
        target_dir,
        hostname,
        max_file_size: 1_000_000,
        verbose: args.verbose,
    };
    if args.verbose {
        println!("{apply_args:#?}");
    }
    homux::apply::apply(apply_args)?;
    Ok(())
}

fn add(args: AddArgs) -> Result<()> {
    let source_dir = if let Some(source_dir) = args.source_dir {
        source_dir
    } else {
        get_default_source_directory().context("get default source directory")?
    };
    let add_args = homux::add::AddArgs {
        target_file: args.new_file,
        source_dir,
        verbose: args.verbose,
    };
    if args.verbose {
        println!("{add_args:#?}");
    }
    homux::add::add(add_args)?;
    Ok(())
}

fn get_default_source_directory() -> Result<PathBuf> {
    Ok(
        PathBuf::from(std::env::var("HOME").context("get user home directory from environment")?)
            .join(".local/share/homux/source"),
    )
}

fn get_machine_hostname() -> Result<String> {
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
