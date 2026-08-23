use std::{fs, path::PathBuf};
use colored::*;
use inquire::Text;
use serde::Serialize;
use serde_json::Value;
use crate::cloud::defaults;
use crate::cloud::cli::pack::Pack;
use crate::cloud::utils_temp;
use crate::cloud::utils_temp::ConfigStatus;

#[derive(Serialize, Debug)]
pub struct CapsuleData {
  capsule: Capsule
}

#[derive(Serialize, Debug)]
pub struct Capsule { 
  name: String,
  created_at: String,
  updated_at: String,
  packs: Vec<Pack>,
}

enum CreateNewCapsuleResult {
    Success(PathBuf),
    Error(String),
}

pub fn new(user_input: Option<PathBuf>) {
  let file_path = utils_temp::verify_capsule(user_input);
  match file_path {
    ConfigStatus::Exists(path) => {
      println!("{}", "Cannot create a new Capsule, a nya.json file already exists!".red());
      println!("Location: {}", path.display());
      println!("You can input a different path by running {}", "nya init -o your_path_here".purple());
      println!("Otherwise, remove the existing file first.");
    },
    ConfigStatus::Missing(result) => {
      let result = create_new_capsule_file(result.0);
      match result {
        CreateNewCapsuleResult::Success(path) => {
          println!("Created new Capsule file at: {}", path.display());
        },
        CreateNewCapsuleResult::Error(error) => {
          eprintln!("{}", error);
        }
      }
    }
  }
}

fn create_new_capsule_file(output_path: PathBuf) -> CreateNewCapsuleResult {
  let name = Text::new("What do you want to call this capsule?")
      .prompt();
  let name = match name {
        Ok(name) => name.to_lowercase().replace(" ", "-"),
        Err(e) => return CreateNewCapsuleResult::Error(format!("Failed to read capsule name: {}", e)),
    };
  
  let capsule: CapsuleData = CapsuleData {
    capsule: Capsule {
        name,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        packs: Vec::new()
    }
  };

  let capsule_string_result = serde_json::to_string_pretty(&capsule);
  let capsule_string = match capsule_string_result {
    Ok(c) => c,
    Err(e) => {
      return CreateNewCapsuleResult::Error(e.to_string())
    }
  };
    
  let file_path = get_capsule_path(output_path);

  if let Some(parent) = file_path.parent() {
      fs::create_dir_all(parent).expect("Failed to create directories");
  }

  fs::write(&file_path, capsule_string).expect("Failed to write capsule file");

  CreateNewCapsuleResult::Success(file_path)

}

pub fn check(path: Option<PathBuf>){
  let exists = utils_temp::verify_capsule(path);
  match exists {
    ConfigStatus::Exists(path) => {
      let current_capsule = read_capsule_file(path);
      println!("{}", "A Capsule file exists in this directory.".green());
      if let Some(capsule) = current_capsule {
        println!("Capsule name: {}", capsule["capsule"]["name"]);
      }
    },
    ConfigStatus::Missing(_) => {
      println!("{}", "No Capsule file found in this directory.".yellow());
    }
  }
}

fn get_capsule_path(path: PathBuf) -> PathBuf {
  if path.display().to_string().contains(".nya/nya.json") {
    return path;
  }

  let full_path = path.join(defaults::CAPSULE_DEFAULT_FILE_DIR_AND_NAME);
  full_path
}

pub fn read_capsule_file(path: PathBuf) -> Option<Value> {
  let file_path = get_capsule_path(path);
  let content = fs::read_to_string(file_path).ok()?;
  serde_json::from_str(&content).ok()
}