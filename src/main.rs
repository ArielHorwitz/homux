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
    /// Validate and print configuration and arguments (WARNING: will print secrets)
    Validate,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = homux::config::Config::new(args.generate_missing).context("get config")?;
    match args.operation {
        Operation::Apply(args) => homux::apply::apply(&args, &config).context("apply")?,
        Operation::Add(args) => homux::add::add(&args, &config).context("add")?,
        Operation::Validate => println!("{config:#?}\n{args:#?}"),
    }
    Ok(())
}
