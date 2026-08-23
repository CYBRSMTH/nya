use nya_cloud::cloud::args::{BaseCommands, CapsuleCommands, Cli, Commands, PackCommands};
use clap::Parser;
use nya_cloud::cloud::cli::{
  base, capsule, init, pack, ship
};

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  match cli.command {
    Commands::Init { output } => { init::run(output) },
    Commands::Base { command } => match command {
      BaseCommands::Build { config }=> { base::build(config).await },
      BaseCommands::Destroy { config }=> { base::destroy(config).await }
    },
    Commands::Capsule { command } => match command {
      CapsuleCommands::New { config } => { capsule::new(config) }
    },
    Commands::Pack { command } => match command {
      PackCommands::New { capsule } => { pack::new(capsule) },
    },
    Commands::Ship { config, location } => { ship::run(config, location).await },
  }
}