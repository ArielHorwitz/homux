use anyhow::{Context, Result};
use clap::Parser;
use homux::{
    config::{generate_default_configuration_file, Config},
    files::get_home_dir,
};
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
    /// Custom configuration file path [default: ~/.config/homux/config.toml]
    #[arg(long)]
    config_file: Option<PathBuf>,
}

#[derive(Debug, Parser)]
enum Operation {
    /// Apply a source directory
    Apply(ApplyArgs),
    /// Add a new file to the source directory
    Add(AddArgs),
    /// Print info
    Print(PrintArgs),
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
    /// Print more verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
}

#[derive(Debug, Parser)]
struct AddArgs {
    /// New file
    #[arg()]
    new_file: PathBuf,
    /// Source directory to add to
    #[arg(short = 's', long)]
    source_dir: Option<PathBuf>,
    /// File relative base path [default: from home directory]
    #[arg(short = 'b', long)]
    relative_base: Option<PathBuf>,
}

#[derive(Debug, Parser)]
struct PrintArgs {
    /// Value to print
    #[arg(value_enum, default_value_t = PrintValue::Source)]
    value: PrintValue,
}

#[derive(Debug, clap::ValueEnum, Clone, Copy)]
enum PrintValue {
    /// Default source directory
    #[clap(name = "source")]
    Source,
}

fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);
    generate_default_configuration_file().context("generate default configuration file")?;
    let config = homux::config::parse_configuration_file(args.config_file)?;
    match args.operation {
        Operation::Apply(args) => apply(args, config)?,
        Operation::Add(args) => add(args, config)?,
        Operation::Print(args) => print_operation(args, config)?,
    }
    Ok(())
}

fn apply(args: ApplyArgs, config: Config) -> Result<()> {
    let source_dir = args.source_dir.unwrap_or(config.source);
    let target_dir = if args.dry_run {
        PathBuf::from(String::from("/tmp/homux.dry_run"))
    } else {
        get_home_dir().context("get home directory")?
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
    dbg!(&apply_args);
    homux::apply::apply(apply_args)?;
    Ok(())
}

fn add(args: AddArgs, config: Config) -> Result<()> {
    let source_dir = args.source_dir.unwrap_or(config.source);
    let target_base = if let Some(relative_base) = args.relative_base {
        relative_base
    } else {
        get_home_dir().context("get default target base")?
    };
    let add_args = homux::add::AddArgs {
        target_file: args.new_file,
        source_dir,
        target_base,
    };
    dbg!(&add_args);
    homux::add::add(add_args)?;
    Ok(())
}

fn print_operation(args: PrintArgs, config: Config) -> Result<()> {
    match args.value {
        PrintValue::Source => println!("{}", config.source.display()),
    }
    Ok(())
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
