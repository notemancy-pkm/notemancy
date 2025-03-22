// src/vectorize.rs
use hddb::core::{create_store, dump_store};
use notemancy_core::ai::sentence_transformer::generate_embedding;
use notemancy_core::crud::read_note;
use notemancy_core::utils::list_notes;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use tch::Tensor;

/// Vectorizes all notes in a vault by generating embeddings using sentence transformers
/// and storing them in a vector store using hddb.
///
/// # Parameters
///
/// - `vault_name`: The name of the vault to vectorize.
///
/// # Returns
///
/// Returns `Ok(())` if the vectorization is successful, otherwise returns an error.
pub async fn vectorize_vault(vault_name: &str) -> Result<(), Box<dyn Error>> {
    println!("Vectorizing notes in vault '{}'...", vault_name);

    // Get the config directory for storing the vector store
    let conf_dir = env::var("NOTEMANCY_CONF_DIR")
        .map_err(|_| "Environment variable NOTEMANCY_CONF_DIR is not set")?;

    // Create the store name
    let store_name = format!("{}_vectors", vault_name);

    // Check if the vector store already exists and remove it
    let store_path = Path::new(&conf_dir).join(format!("{}.bin", store_name));
    if store_path.exists() {
        println!("Removing existing vector store: {}", store_path.display());
        fs::remove_file(&store_path)?;
    }

    // Get all notes from the vault
    let notes = list_notes(vault_name)?;
    if notes.is_empty() {
        println!("No notes found in vault '{}'", vault_name);
        return Ok(());
    }

    println!("Found {} notes", notes.len());

    // Prepare vectors for storing note data and embeddings
    let mut note_ids: Vec<String> = Vec::with_capacity(notes.len());
    let mut embeddings: Vec<Tensor> = Vec::with_capacity(notes.len());

    // Process each note
    for note in notes {
        println!("Processing note: {}", note.relpath);

        // Read the note content without frontmatter
        let content = read_note(vault_name, &note.relpath, false)?;

        // Generate embedding for the note content
        let note_embeddings = generate_embedding(&content)?;

        if note_embeddings.is_empty() {
            println!(
                "  Warning: No embedding generated for note {}",
                note.relpath
            );
            continue;
        }

        // The embedding should be a single vector for the entire note
        let embedding = &note_embeddings[0];

        // Convert to tensor
        let tensor = Tensor::f_from_slice(embedding)
            .map_err(|e| format!("Failed to create tensor: {}", e))?;

        // Store the note ID and embedding
        note_ids.push(note.relpath.clone());
        embeddings.push(tensor);
    }

    if embeddings.is_empty() {
        println!("No embeddings were generated");
        return Ok(());
    }

    // Stack all embeddings into a single tensor
    let stacked_embeddings = Tensor::stack(&embeddings, 0);

    // Create a store with the embeddings
    let mut store = create_store(stacked_embeddings);

    // Update store IDs to match note relpaths
    for (i, id) in note_ids.iter().enumerate() {
        store.index_to_id.insert(i, id.clone());
        store.id_to_index.insert(id.clone(), i);
    }

    // Save the vector store - explicitly handle the error
    match dump_store(&conf_dir, &store_name, &store).await {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed to dump store: {}", e).into()),
    }

    println!("Vector store created and saved as '{}'", store_name);
    println!("Location: {}", store_path.display());

    Ok(())
}
