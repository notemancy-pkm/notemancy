mod config;
mod crud;
mod edit;
mod picker;
mod vectorize;

use config::{init_config, set_default_vault};
use crud::new_note;
use edit::edit_note;
use notemancy_core::config::get_vault_dir;
use picker::pick_note;
use std::env;
use std::fs;
use std::process;

/// Helper function to get the default vault name from configuration.
fn get_default_vault() -> Result<String, Box<dyn std::error::Error>> {
    let conf_dir = env::var("NOTEMANCY_CONF_DIR")?;
    let default_path = std::path::Path::new(&conf_dir).join("default_vault.txt");
    if default_path.exists() {
        let vault_name = fs::read_to_string(default_path)?;
        let trimmed = vault_name.trim().to_string();
        if trimmed.is_empty() {
            return Err("No default vault set".into());
        }
        Ok(trimmed)
    } else {
        Err("No default vault set".into())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // If no command is provided, run the nucleo-picker based note selector.
    if args.len() == 1 {
        if let Err(err) = pick_note() {
            eprintln!("Error picking note: {}", err);
            process::exit(1);
        }
        return;
    }

    match args[1].as_str() {
        "init" => {
            if let Err(err) = init_config() {
                eprintln!("Error initializing notemancy: {}", err);
                process::exit(1);
            }
        }
        "set" => {
            if args.len() < 3 {
                eprintln!("Usage: notemancy set <vault_name>");
                process::exit(1);
            }
            let vault_name = &args[2];
            if let Err(err) = set_default_vault(vault_name) {
                eprintln!("Error setting default vault: {}", err);
                process::exit(1);
            }
            println!("Default vault set to {}", vault_name);
        }
        "n" => {
            if let Err(err) = new_note() {
                eprintln!("Error creating new note: {}", err);
                process::exit(1);
            }
        }
        "-e" => {
            // If a vault is specified with an '@' prefix (e.g., "-e @vault_name"), use that vault.
            let vault_arg = if args.len() >= 3 && args[2].starts_with('@') {
                Some(args[2].trim_start_matches('@').to_string())
            } else {
                None
            };
            if let Err(err) = edit_note(vault_arg) {
                eprintln!("Error editing note: {}", err);
                process::exit(1);
            }
        }
        "cd" => {
            if args.len() < 3 {
                eprintln!("Usage: notemancy cd <vault_name>");
                process::exit(1);
            }
            let vault_name = &args[2];
            match get_vault_dir(vault_name) {
                Ok(dir) => println!("{}", dir),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            }
        }
        "vectorize" => {
            // Determine which vault to use
            let vault = if args.len() >= 3 {
                args[2].clone()
            } else {
                // Use the default vault
                match get_default_vault() {
                    Ok(vault) => vault,
                    Err(err) => {
                        eprintln!(
                            "Error: {}; please specify a vault with 'notemancy vectorize <vault_name>'",
                            err
                        );
                        process::exit(1);
                    }
                }
            };

            // Create a new tokio runtime for async operations
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(err) => {
                    eprintln!("Error creating async runtime: {}", err);
                    process::exit(1);
                }
            };

            // Run vectorization
            if let Err(err) = rt.block_on(vectorize::vectorize_vault(&vault)) {
                eprintln!("Error vectorizing vault: {}", err);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}. Usage: notemancy <command>", args[1]);
            process::exit(1);
        }
    }
}
