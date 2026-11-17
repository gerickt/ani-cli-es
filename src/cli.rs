use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "ani-cli-es",
    version = "5.2.0-es",
    about = "Cliente TUI y CLI en Rust para explorar, ver y descargar anime en español desde JKAnime",
    after_help = "Ejemplos de uso:
  ani-cli-es                  # Abre el menú interactivo Hub (Ratatui TUI)
  ani-cli-es 'solo leveling'  # Abre la TUI con resultados de búsqueda
  ani-cli-es -E               # Ver cartelera de estrenos
  ani-cli-es -G               # Explorar animes por géneros
  ani-cli-es -c               # Ver historial y continuar reproducción
  ani-cli-es -B               # Explorar biblioteca de descargas offline
  ani-cli-es -d -e 1 naruto   # Descargar episodio 1 directamente"
)]
pub struct Cli {
    /// Término de búsqueda de anime
    pub query: Option<String>,

    /// Ver cartelera de estrenos de hoy y recientes
    #[arg(short = 'E', long = "estrenos")]
    pub estrenos: bool,

    /// Explorar animes por géneros y categorías
    #[arg(short = 'G', long = "generos")]
    pub generos: bool,

    /// Explorar y reproducir descargas locales offline
    #[arg(short = 'B', long = "biblioteca")]
    pub biblioteca: bool,

    /// Ver historial interactivo y continuar viendo
    #[arg(short = 'c', long = "continue")]
    pub continue_history: bool,

    /// Descargar episodio en lugar de reproducirlo
    #[arg(short = 'd', long = "download")]
    pub download: bool,

    /// Borrar historial de reproducción
    #[arg(short = 'D', long = "delete")]
    pub delete_history: bool,

    /// Especificar número de episodio a reproducir/descargar
    #[arg(short = 'e', long = "episode", alias = "range")]
    pub episode: Option<u32>,

    /// Especificar calidad de video (best, 1080p, 720p, 480p, worst)
    #[arg(short = 'q', long = "quality")]
    pub quality: Option<String>,

    /// Usar VLC como reproductor de medios en lugar de mpv
    #[arg(short = 'v', long = "vlc")]
    pub vlc: bool,

    /// Usar ani-skip para saltar la intro del episodio (mpv)
    #[arg(long = "skip")]
    pub skip: bool,

    /// Mostrar ficha técnica y sinopsis de un anime
    #[arg(long = "preview")]
    pub preview: bool,
}
