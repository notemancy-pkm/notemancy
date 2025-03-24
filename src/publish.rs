// src/publish.rs

use notemancy_core::config::{get_vault_dir, read_config};
use notemancy_core::crud::read_note;
use notemancy_core::utils::list_notes;
use reqwest::blocking::Client;
use serde::Serialize;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct UploadNoteRequest {
    relpath: String,
    content: String,
}

pub fn publish_notes() -> Result<(), Box<dyn Error>> {
    // Read the default vault name from default_vault.txt
    let conf_dir = env::var("NOTEMANCY_CONF_DIR")?;
    let default_vault_path = Path::new(&conf_dir).join("default_vault.txt");
    let vault = if default_vault_path.exists() {
        let s = fs::read_to_string(default_vault_path)?;
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            return Err("No default vault set".into());
        }
        trimmed
    } else {
        return Err("No default vault set".into());
    };

    // Read the publish_url from the configuration (config.yaml)
    let config = read_config()?;
    let publish_url = config
        .get("publish_url")
        .and_then(|v| v.as_str())
        .ok_or("publish_url not found in config")?;
    let endpoint = if publish_url.ends_with('/') {
        format!("{}notes/upload", publish_url)
    } else {
        format!("{}/notes/upload", publish_url)
    };

    // List all notes in the vault
    let notes = list_notes(&vault)?;
    if notes.is_empty() {
        println!("No notes found in vault '{}'", vault);
        return Ok(());
    }
    println!("Found {} notes in vault '{}'", notes.len(), vault);

    let client = Client::new();
    let mut failures = Vec::new();

    // For each note, read its full content and post it to the publish endpoint
    for note in notes {
        println!("Uploading note: {}", note.relpath);
        let content = read_note(&vault, &note.relpath, true)?;
        let req_body = UploadNoteRequest {
            relpath: note.relpath.clone(),
            content,
        };
        let res = client.post(&endpoint).json(&req_body).send();

        match res {
            Ok(resp) => {
                if !resp.status().is_success() {
                    println!("Failed to upload {}: HTTP {}", note.relpath, resp.status());
                    failures.push(note.relpath);
                } else {
                    println!("Uploaded {} successfully", note.relpath);
                }
            }
            Err(e) => {
                println!("Error uploading {}: {}", note.relpath, e);
                failures.push(note.relpath);
            }
        }
    }

    if !failures.is_empty() {
        println!("The following notes failed to upload: {:?}", failures);
    } else {
        println!("All notes uploaded successfully!");
    }

    Ok(())
}
