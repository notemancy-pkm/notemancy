use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Initializes the notemancy configuration.
///
/// This function checks for the configuration directory specified by the
/// `NOTEMANCY_CONF_DIR` environment variable. If the directory does not exist,
/// it is created. Then it checks for the existence of `config.yaml` within that
/// directory; if the file does not exist, an empty `config.yaml` is created.
/// Finally, it opens the configuration file in the default editor as configured
/// in the shell (using the `EDITOR` environment variable, defaulting to "vi").
pub fn init_config() -> Result<(), Box<dyn Error>> {
    // Retrieve the configuration directory from the environment variable.
    let conf_dir = env::var("NOTEMANCY_CONF_DIR")
        .map_err(|_| "Environment variable NOTEMANCY_CONF_DIR is not set")?;
    let conf_path = Path::new(&conf_dir);

    // Ensure the configuration directory exists.
    if !conf_path.exists() {
        fs::create_dir_all(&conf_path)?;
    }

    // Define the path to config.yaml.
    let config_file = conf_path.join("config.yaml");

    // If config.yaml does not exist, create an empty file.
    if !config_file.exists() {
        fs::write(&config_file, "")?;
    }

    // Open the config.yaml file in the default editor.
    // Use the EDITOR environment variable, or default to "vi".
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    Command::new(editor).arg(&config_file).status()?;

    Ok(())
}

/// Sets the default vault by writing the given vault name to a file in the configuration directory.
/// The configuration directory is determined by the `NOTEMANCY_CONF_DIR` environment variable.
pub fn set_default_vault(vault_name: &str) -> Result<(), Box<dyn Error>> {
    // Retrieve the configuration directory.
    let conf_dir = env::var("NOTEMANCY_CONF_DIR")
        .map_err(|_| "Environment variable NOTEMANCY_CONF_DIR is not set")?;
    let conf_path = Path::new(&conf_dir);

    // Ensure the configuration directory exists.
    if !conf_path.exists() {
        fs::create_dir_all(&conf_path)?;
    }

    // Define the file path to store the default vault.
    let default_vault_path = conf_path.join("default_vault.txt");

    // Write the vault name into the file.
    fs::write(default_vault_path, vault_name)?;

    Ok(())
}
