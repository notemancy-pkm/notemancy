use notemancy_core::config::get_vault_dir;
use notemancy_core::utils::list_notes;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Opens the fzf selector for notes in the specified vault (or default vault).
/// If a vault is provided in `vault_option`, that vault is used;
/// otherwise, the default vault is read from the configuration directory.
/// After the user selects a note via fzf (formatted as "title | relative_path"),
/// this function constructs the absolute path to the note and opens it in the default editor.
pub fn edit_note(vault_option: Option<String>) -> Result<(), Box<dyn Error>> {
    // Determine which vault to use.
    let vault = if let Some(v) = vault_option {
        v
    } else {
        // Look for a default vault in the configuration directory.
        let conf_dir = env::var("NOTEMANCY_CONF_DIR")
            .map_err(|_| "Environment variable NOTEMANCY_CONF_DIR is not set")?;
        let default_path = std::path::Path::new(&conf_dir).join("default_vault.txt");
        if default_path.exists() {
            let vault_name = fs::read_to_string(default_path)?;
            let trimmed = vault_name.trim().to_string();
            if !trimmed.is_empty() {
                trimmed
            } else {
                return Err("No default vault set; please specify one with '@vault_name'".into());
            }
        } else {
            return Err("No default vault set; please specify one with '@vault_name'".into());
        }
    };

    // Retrieve the list of notes from the vault.
    let notes = list_notes(&vault)?;
    if notes.is_empty() {
        return Err(format!("No notes found in vault '{}'", vault).into());
    }

    // Format each note as "title | relative_path".
    let lines: Vec<String> = notes
        .iter()
        .map(|note| format!("{} | {}", note.title, note.relpath))
        .collect();
    let input = lines.join("\n");

    // Spawn fzf, feeding it the list of notes via stdin and capturing its stdout.
    let mut fzf = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let stdin = fzf.stdin.as_mut().ok_or("Failed to open fzf stdin")?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = fzf.wait_with_output()?;
    if !output.status.success() {
        return Err("fzf did not exit successfully or no selection was made".into());
    }
    let selection = String::from_utf8(output.stdout)?.trim().to_string();
    if selection.is_empty() {
        return Err("No note selected".into());
    }

    // The selection is expected to be in the format "title | relative_path".
    let parts: Vec<&str> = selection.split(" | ").collect();
    if parts.len() < 2 {
        return Err("Invalid selection format from fzf".into());
    }
    let rel_path = parts[1].trim();

    // Construct the full file path by joining the vault directory with the relative path.
    let vault_dir = get_vault_dir(&vault)?;
    let full_path = PathBuf::from(vault_dir).join(rel_path);

    // Instead of opening the file in an editor, print the absolute path to stdout.
    println!("{}", full_path.display());
    // Open the file in the default editor.
    // let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    // Command::new(editor).arg(full_path).status()?;

    Ok(())
}
