// src/crud.rs

use inquire::Text;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Import the create_note method and get_vault_dir from the notemancy-core library.
use notemancy_core::config::get_vault_dir;
use notemancy_core::crud::create_note;

/// A helper function that sanitizes a title string into a valid file name.
/// This replicates the logic in the core library.
fn sanitize_title(title: &str) -> String {
    let lower = title.trim().to_lowercase();
    let mapped: String = lower
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    mapped
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Creates a new note using interactive prompts.
///
/// This function prompts the user for the note title, vault (unless a default vault is set),
/// and an optional project. It then creates the note using the core's create_note function
/// and finally opens the newly created file in the default editor (determined by the EDITOR environment variable).
pub fn new_note() -> Result<(), Box<dyn Error>> {
    // Check if a default vault is set in the config directory.
    let default_vault = if let Ok(conf_dir) = env::var("NOTEMANCY_CONF_DIR") {
        let default_path = Path::new(&conf_dir).join("default_vault.txt");
        if default_path.exists() {
            let vault_name = fs::read_to_string(default_path)?;
            let trimmed = vault_name.trim().to_string();
            if !trimmed.is_empty() {
                Some(trimmed)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Prompt for the note title.
    let title = Text::new("Enter note title:").prompt()?;

    // Use the default vault if set; otherwise, prompt the user for vault name.
    let vault = if let Some(vault_name) = default_vault {
        vault_name
    } else {
        Text::new("Enter vault name:").prompt()?
    };

    // Prompt for an optional project; if nothing is provided, default to an empty string.
    let project = Text::new("Enter project (optional):")
        .with_default("")
        .prompt()?;

    // Create the note using the core library's create_note function.
    create_note(&vault, &project, &title)?;

    // Reconstruct the file path for the newly created note.
    let vault_dir = get_vault_dir(&vault)?;
    let project_path = if project.is_empty() {
        PathBuf::from(vault_dir)
    } else {
        Path::new(&vault_dir).join(&project)
    };
    let sanitized_title = sanitize_title(&title);
    let file_name = format!("{}.md", sanitized_title);
    let file_path = project_path.join(file_name);

    // Open the newly created note in the default editor.
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    Command::new(editor).arg(file_path).status()?;

    Ok(())
}
