mod config;
mod crud;
mod edit;
mod picker;

use config::{init_config, set_default_vault};
use crud::new_note;
use edit::edit_note;
use notemancy_core::config::get_vault_dir;
use picker::pick_note;
use std::env;
use std::process;

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
        _ => {
            eprintln!("Unknown command: {}. Usage: notemancy <command>", args[1]);
            process::exit(1);
        }
    }
}
