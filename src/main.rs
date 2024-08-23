use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(name = "homux")]
#[clap(about = "Synchronize your home directory across host machines.")]
#[clap(author = "https://ariel.ninja")]
#[clap(version)]
struct Args {
    /// Operation
    #[command(subcommand)]
    operation: Operation,
    /// Custom location for config file
    #[arg(long)]
    config_file: Option<std::path::PathBuf>,
    /// Generate missing configuration file
    #[arg(long)]
    generate_missing: bool,
}

#[derive(Debug, Clone, Parser)]
enum Operation {
    /// Apply a source directory
    Apply(homux::apply::Args),
    /// Add a new file to the source directory
    Add(homux::add::Args),
    /// Validate configuration
    Validate(ValidateArgs),
}

#[derive(Debug, Clone, Copy, Parser)]
struct ValidateArgs {
    /// Print configuration (WARNING: will print secrets)
    #[arg(long, short = 'p')]
    print: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = homux::config::Config::new(args.config_file, args.generate_missing)
        .context("get config")?;
    match args.operation {
        Operation::Apply(args) => homux::apply::apply(&args, &config).context("apply")?,
        Operation::Add(args) => homux::add::add(&args, &config).context("add")?,
        Operation::Validate(args) => {
            if args.print {
                println!("{config:#?}");
            }
        }
    }
    Ok(())
}
