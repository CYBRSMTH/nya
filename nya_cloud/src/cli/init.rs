use std::fs;
use std::path::PathBuf;
use crate::cli::utils::{ConfigStatus, verify_base_config};
use colored::*;

pub fn run(user_input: Option<PathBuf>) {
  match verify_base_config(user_input) {
    ConfigStatus::Exists(path) => {
      println!("{}", "Cannot initialize Base Config, file already exists!".red());
      println!("Location: {}", path.display());
      println!("You can input a different path by running {}", "nya init -o your_path_here".purple());
      println!("Otherwise, remove the existing file first.");
    },
    ConfigStatus::Missing((path, _)) => {
      if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
          println!("Failed to create directory at {}: {}", parent.display(), e);
          return;
        }
      }

      if let Err(e) = fs::write(&path, include_str!("../ops/init/initial_config.json")) {
        println!("Failed to create config file at {}: {}", path.display(), e);
        return;
      }

      println!("{}", "Created Nya base config template".green());
      println!("Location: {}", path.display());
      println!();
      println!("{}", "Next steps:".cyan());
      println!("1. Edit the config file and fill in your infrastructure details");
      println!("2. Run: {}", "nya base build".bright_purple());
    }
  }
}
