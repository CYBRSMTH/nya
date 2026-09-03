use std::fs;
use std::path::PathBuf;
use colored::*;
use inquire::{Select, Text};
use serde::Serialize;
use serde_json::json;
use tera::{Context, Tera};
use crate::cli::capsule::read_capsule_file;
use crate::cli::utils::{ConfigStatus, verify_capsule};

#[derive(Serialize, Debug)]
pub struct Pack {
  name: String,
  pack_type: String,
  location: String,
  created_at: String,
  updated_at: String,
  port: Option<u16>,
}

pub fn new(capsule: Option<PathBuf>) {
  let capsule_path_buf = verify_capsule(capsule);
  let capsule_path = match capsule_path_buf {
    ConfigStatus::Exists(path) => path,
    ConfigStatus::Missing(result)=> {
      println!("{}", "No Capsule found. Please navigate to your Capsule".yellow());
      println!("You can create one by running {}{}", "nya capsule new -c ".purple(), result.0.display().to_string().purple());
      return;
    }
  };

  let mut capsule = read_capsule_file(capsule_path.clone()).unwrap();
  let current_packs: Vec<String> = capsule["capsule"]["packs"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|p| p["name"].as_str())
        .map(String::from)
        .collect();

  let name = match Text::new("What do you want to name this pack?").prompt() {
      Ok(n) => n.to_lowercase().trim_end().replace(" ", "-"),
      Err(_) => {
          println!("{}", "Cancelled".yellow());
          return;
      }
  };

  if current_packs.contains(&name) {
      println!("{}", format!("Pack '{}' already exists!", name).red());
      return;
  }

  let pack_type = match Select::new(
      "What type of pack?",
      vec!["frontend", "backend"]
  ).prompt() {
      Ok(t) => t,
      Err(_) => return,
  };

  let mut port: u16 = 0;
  if pack_type != "frontend" {
    port = match Text::new("What port do you want for your backend?")
    .with_default("80")
    .prompt() {
        Ok(n) => n.parse::<u16>().unwrap_or(80),
        Err(_) => {
            println!("{}", "Cancelled".yellow());
            return;
        }
    };
  };

  let pack_values_content = match pack_type {
    "frontend" => include_str!("../ops/pack/values.frontend.yaml"),
    "backend" => include_str!("../ops/pack/values.backend.yaml"),
    _ => { 
      print!("There was an issue with getting the values for your pack. Please try again.");
      return;
    },
  };

  let mut pack_path_buf = capsule_path.clone();
  pack_path_buf.pop();
  pack_path_buf.pop();
  pack_path_buf.push(&name);
  
  let pack: Pack = Pack {
    name: name.clone(),
    created_at: chrono::Utc::now().to_rfc3339(),
    updated_at: chrono::Utc::now().to_rfc3339(),
    pack_type: String::from(pack_type),
    port: if pack_type == "frontend" { None } else {  Some(port) },
    location: name.clone()  // Just the name, relative to capsule root
  };
  
  let pack_value = serde_json::to_value(&pack).unwrap();
  let pack_context = Context::from_serialize(&pack).unwrap();
  let hydrated_pack_values_content = Tera::one_off(pack_values_content, &pack_context, true);
  
  capsule["capsule"]["packs"].as_array_mut().unwrap().push(pack_value);
  capsule["capsule"]["updated_at"] = json!(chrono::Utc::now().to_rfc3339());

  if let Err(e) = fs::write(&capsule_path, serde_json::to_string_pretty(&capsule).unwrap()) {
    eprintln!("{}", format!("Failed to save capsule: {}", e).red());
    return;
  }

  let dockerfile_path = pack_path_buf.clone().join("Dockerfile");
  let values_path = pack_path_buf.clone().join("values.yaml");

  if let Err(e) = fs::create_dir_all(&pack_path_buf) {
    println!("Failed to create directories: {} \n {}", pack_path_buf.display(), e);
    return
  }

  if let Err(e) = fs::write(dockerfile_path, include_str!("../ops/pack/Dockerfile")) {
      println!("Failed to create config file at {}: {}", &pack_path_buf.display(), e);
      return;
  }

  if let Err(e) = fs::write(values_path, hydrated_pack_values_content.unwrap()) {
      println!("Failed to create config file at {}: {}", &pack_path_buf.display(), e);
      return;
  }

  println!("{}", format!("✓ Created pack: {}", name).green());
  println!("Location: {}", &pack_path_buf.display());
  //TODO: single pack deployments
  //println!("Edit your Dockerfile, then run: {}", "nya pack deploy".purple());
}