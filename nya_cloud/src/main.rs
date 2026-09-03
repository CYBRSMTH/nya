mod cli;
mod ops;
use crate::cli::args::{BaseCommands, CapsuleCommands, Cli, Commands, PackCommands};
use clap::Parser;
use crate::cli::{
  base, capsule, init, pack, ship
};
use anyhow::{Result};

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Commands::Init { output } => { init::run(output) },
    Commands::Base { command } => match command {
      BaseCommands::Build { config }=> { base::build(config).await? },
      BaseCommands::Destroy { config }=> { base::destroy(config).await? }
    },
    Commands::Capsule { command } => match command {
      CapsuleCommands::New { config } => { capsule::new(config) }
    },
    Commands::Pack { command } => match command {
      PackCommands::New { capsule } => { pack::new(capsule) },
    },
    Commands::Ship { config, location } => { ship::run(config, location).await? },
  }
  Ok(())
}