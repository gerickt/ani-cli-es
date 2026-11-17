use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct LocalDownload {
    pub file_name: String,
    pub path: PathBuf,
    pub size_formatted: String,
}

pub fn scan_library<P: AsRef<Path>>(dir: P) -> Vec<LocalDownload> {
    let mut files = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "mp4" || ext_lower == "mkv" || ext_lower == "webm" {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let size_formatted = if let Ok(meta) = entry.metadata() {
                        let bytes = meta.len();
                        if bytes > 1024 * 1024 * 1024 {
                            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
                        } else {
                            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                        }
                    } else {
                        "Desc.".to_string()
                    };

                    files.push(LocalDownload {
                        file_name,
                        path,
                        size_formatted,
                    });
                }
            }
        }
    }

    files.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    files
}
