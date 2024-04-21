use anyhow::{Context, Result};
use clap::Parser;
use homux::files::get_home_dir;
use homux::{get_machine_hostname, get_default_source_directory};
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
    /// Show more verbose output
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
    /// File relative base path [default from home directory]
    #[arg(short = 'b', long)]
    relative_base: Option<PathBuf>,
    /// Show more verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
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
    match args.operation {
        Operation::Apply(args) => apply(args)?,
        Operation::Add(args) => add(args)?,
        Operation::Print(args) => print_operation(args)?,
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
    let target_base = if let Some(relative_base) = args.relative_base {
        relative_base
    } else {
        get_home_dir().context("get default target base")?
    };
    let add_args = homux::add::AddArgs {
        target_file: args.new_file,
        source_dir,
        target_base,
        verbose: args.verbose,
    };
    if args.verbose {
        println!("{add_args:#?}");
    }
    homux::add::add(add_args)?;
    Ok(())
}

fn print_operation(args: PrintArgs) -> Result<()> {
    match args.value {
        PrintValue::Source => println!("{}", get_default_source_directory()?.display()),
    }
    Ok(())
}
