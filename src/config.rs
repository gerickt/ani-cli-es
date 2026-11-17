use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub player: String,
    pub download_dir: PathBuf,
    pub quality: String,
    pub history_file: PathBuf,
    pub skip_intro: bool,
}

impl Config {
    pub fn load() -> Self {
        let player = std::env::var("ANI_CLI_PLAYER").unwrap_or_else(|_| {
            if which::which("mpv").is_ok() {
                "mpv".to_string()
            } else if which::which("vlc").is_ok() {
                "vlc".to_string()
            } else if which::which("iina").is_ok() {
                "iina".to_string()
            } else {
                "mpv".to_string()
            }
        });

        let download_dir = std::env::var("ANI_CLI_DOWNLOAD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::download_dir().unwrap_or_else(|| PathBuf::from("."))
            });

        let quality = std::env::var("ANI_CLI_QUALITY").unwrap_or_else(|_| "best".to_string());

        let state_dir = dirs::state_dir()
            .map(|d| d.join("ani-cli-es"))
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".local")
                    .join("state")
                    .join("ani-cli-es")
            });

        let history_file = state_dir.join("history");

        Self {
            player,
            download_dir,
            quality,
            history_file,
            skip_intro: false,
        }
    }
}

// Minimal helper to check if a command exists in PATH
mod which {
    use std::path::PathBuf;

    pub fn which(name: &str) -> Result<PathBuf, ()> {
        if let Ok(paths) = std::env::var("PATH") {
            for p in std::env::split_paths(&paths) {
                let full = p.join(name);
                if full.is_file() {
                    return Ok(full);
                }
            }
        }
        Err(())
    }
}
