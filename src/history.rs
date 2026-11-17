use anyhow::Result;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub slug: String,
    pub title: String,
    pub episode: u32,
}

pub fn load_history<P: AsRef<Path>>(path: P) -> Vec<HistoryEntry> {
    let mut entries = Vec::new();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return entries,
    };

    let reader = BufReader::new(file);
    for line in reader.lines().map_while(Result::ok) {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let slug = parts[0].trim().to_string();
            let title = parts[1].trim().to_string();
            let ep: u32 = parts[2].trim().parse().unwrap_or(1);
            if !slug.is_empty() {
                entries.push(HistoryEntry {
                    slug,
                    title,
                    episode: ep,
                });
            }
        }
    }

    // Most recent entries first
    entries.reverse();
    // Deduplicate by slug keeping most recent
    let mut unique = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for entry in entries {
        if seen.insert(entry.slug.clone()) {
            unique.push(entry);
        }
    }

    unique
}

pub fn save_entry<P: AsRef<Path>>(
    path: P,
    slug: &str,
    title: &str,
    episode: u32,
) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        create_dir_all(parent)?;
    }

    let mut existing: Vec<String> = Vec::new();
    if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let parts: Vec<&str> = line.split('\t').collect();
            if !parts.is_empty() && parts[0].trim() != slug {
                existing.push(line);
            }
        }
    }

    // Add new entry to the end
    existing.push(format!("{}\t{}\t{}", slug, title, episode));

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;

    for line in existing {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

pub fn clear_history<P: AsRef<Path>>(path: P) -> Result<()> {
    if path.as_ref().exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
