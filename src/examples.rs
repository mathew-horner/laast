use std::path::PathBuf;

use anyhow::{anyhow, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::fs::{self, DirEntry};
use tokio::task;

use crate::{Laast, Language};

/// Read all of the language Laasts for the given example from the examples directory.
pub async fn read(example_name: &str) -> Result<Vec<Laast>> {
    let directory = PathBuf::from("examples").join(example_name);
    if !(directory.exists() || directory.is_dir()) {
        return Err(anyhow!("example \"{example_name}\" does not exist."));
    }

    let mut read_dir = fs::read_dir(&directory).await?;

    // NOCHECKIN: Should we be using a JoinSet here?
    let futs = FuturesUnordered::new();

    // Propagate if we fail to get the next entry, we don't want to spin here indefinitely.
    while let Some(entry) = read_dir.next_entry().await? {
        futs.push(process_entry(entry));
    }

    let laasts = futs
        .filter_map(|result| async move {
            match result {
                Ok(laast) => Some(laast),
                Err(error) => {
                    log::warn!("failed to process entry: {error}");
                    None
                }
            }
        })
        .collect()
        .await;

    Ok(laasts)
}

async fn process_entry(entry: DirEntry) -> Result<Laast> {
    if !entry
        .file_type()
        .await
        .map(|ty| ty.is_file())
        .unwrap_or(false)
    {
        return Err(anyhow!("failed to identify entry as a file"));
    }

    let file_name = PathBuf::from(entry.file_name());
    let language = Language::infer_from_filename(&file_name)?;

    let code = fs::read_to_string(entry.path()).await?;

    // Laast::parse can be a (relatively) CPU heavy function, hence the spawn_blocking here.
    task::spawn_blocking(move || Laast::parse(language, &code)).await?
}
