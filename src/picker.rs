use notemancy_core::config::get_vault_dir;
use notemancy_core::utils::{NoteInfo, list_notes};
use nucleo_picker::{PickerOptions, nucleo::Config, render::StrRenderer};
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn pick_note() -> Result<(), Box<dyn Error>> {
    // Determine the default vault from the configuration directory.
    let conf_dir = env::var("NOTEMANCY_CONF_DIR")
        .map_err(|_| "Environment variable NOTEMANCY_CONF_DIR is not set")?;
    let default_path = std::path::Path::new(&conf_dir).join("default_vault.txt");
    let vault = if default_path.exists() {
        let vault_name = fs::read_to_string(&default_path)?;
        let trimmed = vault_name.trim().to_string();
        if trimmed.is_empty() {
            return Err(
                "No default vault set; please set one using 'notemancy set <vault_name>'".into(),
            );
        } else {
            trimmed
        }
    } else {
        return Err(
            "No default vault set; please set one using 'notemancy set <vault_name>'".into(),
        );
    };

    // Retrieve the list of notes from the vault.
    let notes: Vec<NoteInfo> = list_notes(&vault)?;
    if notes.is_empty() {
        return Err(format!("No notes found in vault '{}'", vault).into());
    }

    // Format each note as "title | relative_path".
    let choices: Vec<String> = notes
        .iter()
        .map(|note| format!("{} | {}", note.title, note.relpath))
        .collect();

    // Configure nucleo-picker.
    let mut picker = PickerOptions::default()
        .config(Config::DEFAULT.match_paths())
        .query("")
        .picker(StrRenderer);

    // Push each choice into the picker.
    let injector = picker.injector();
    for choice in choices {
        injector.push(choice);
    }

    // Launch the interactive prompt and capture the selection.
    let selection = picker.pick()?;
    let selected = match selection {
        Some(s) => s,
        None => return Err("No note selected".into()),
    };

    // Parse the selected line (expected format: "title | relative_path").
    let parts: Vec<&str> = selected.split(" | ").collect();
    if parts.len() < 2 {
        return Err("Invalid selection format".into());
    }
    let rel_path = parts[1].trim();

    // Construct the full file path.
    let vault_dir = get_vault_dir(&vault)?;
    let full_path = PathBuf::from(vault_dir).join(rel_path);

    // Open the note in the default editor.
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    std::process::Command::new(editor).arg(full_path).status()?;

    Ok(())
}
