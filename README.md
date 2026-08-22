<h1 align="center">
	ani-cli-es 🇲🇽 🇪🇸
</h1>

<h3 align="center">
Tu experiencia de anime en español definitiva desde la terminal.
</h3>

<p align="center">
  <a href="#instalación"><img src="https://img.shields.io/badge/versión-v5.1.0--es-purple?style=for-the-badge"></a>
  <a href="#instalación"><img src="https://img.shields.io/badge/instalación-rápida-brightgreen?style=for-the-badge"></a>
  <a href="#uso"><img src="https://img.shields.io/badge/uso-fácil-blue?style=for-the-badge"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/licencia-GPLv3-red?style=for-the-badge"></a>
</p>

---

## 📌 Características Principales

- 🇲🇽 🇪🇸 **100% Nativo en Español:** Búsqueda, extracción y transmisión instantánea de anime con subtítulos en español y audio latino.
- 🎨 **TUI Riced & Moderna:** Nueva interfaz con paleta TokyoNight, bordes redondeados y soporte para renderizado de pósters en terminal con **`chafa`**.
- 🌟 **Cartelera de Estrenos en Vivo (`-E` / `--estrenos`):** Consulta directa de los episodios emitidos hoy, ayer y esta semana.
- 🏷️ **Explorador por Géneros y Categorías (`-G` / `--generos`):** Navega por más de 30 categorías (Isekai, Shonen, Seinen, Romance, etc.).
- 🕒 **Historial TUI Robusto (`-c` / `--continue`):** Menú interactivo para continuar donde lo dejaste, seleccionar episodios o volver a ver.
- 📂 **Biblioteca Offline (`-B` / `--biblioteca`):** Explora y reproduce tus episodios descargados localmente sin conexión a internet.
- ⚡ **Extracción Multi-Servidor:** Soporte para failover automático de servidores (HLS `.m3u8` y `.mp4`).
- 🍿 **Multiplataforma y Múltiples Reproductores:** Compatible con `mpv` (con auto-resume), `vlc`, `iina`, `rofi`, `dmenu` y descarga con `yt-dlp` / `ffmpeg`.

---

## 🚀 Instalación

### Requisitos previos
Asegúrate de tener instalados en tu sistema:
* **Obligatorios**: `curl`, `grep`, `sed`, `fzf` (o `rofi` / `dmenu`), `mpv` (o `vlc` / `iina`).
* **Recomendado (Pósters en Terminal)**: `chafa` (`sudo pacman -S chafa` / `sudo apt install chafa` / `brew install chafa`).
* **Descargas**: `yt-dlp` o `ffmpeg`.

### Instalación Rápida (Linux / macOS)

```sh
# 1. Clonar el repositorio
git clone https://github.com/gerickt/ani-cli-es.git
cd ani-cli-es

# 2. Instalar ejecutable y manual
sudo cp ani-cli-es /usr/local/bin/
sudo cp ani-cli-es.1 /usr/local/share/man/man1/ 2>/dev/null || true
```

---

## 🎮 Uso

### Hub Interactivo
Si ejecutas el comando sin argumentos, se abrirá el nuevo **Hub Principal de Navegación**:

```sh
ani-cli-es
```

### Opciones y Atajos

| Comando | Descripción |
| :--- | :--- |
| `ani-cli-es "anime"` | Búsqueda e interfaz interactiva con `fzf` |
| `ani-cli-es -E` | Abre la cartelera de estrenos de la semana |
| `ani-cli-es -G` | Abre el explorador por géneros y categorías |
| `ani-cli-es -B` | Abre la biblioteca de descargas offline |
| `ani-cli-es -c` | Abre el menú interactivo de historial |
| `ani-cli-es -e 1 "cyberpunk edgerunners"` | Reproduce directamente el episodio 1 |
| `ani-cli-es -e "1-5" "naruto"` | Reproduce en maratón los episodios 1 al 5 |
| `ani-cli-es -d -e 1 "solo leveling"` | Descarga el episodio en disco |
| `ani-cli-es --preview "slug"` | Muestra ficha técnica y póster del anime |

---

## 🔄 Historial y Configuración

El historial de reproducción se guarda automáticamente en:
`~/.local/state/ani-cli-es/ani-hsts`

Las carátulas cacheadas se almacenan en:
`~/.cache/ani-cli-es/covers/`

Variables de entorno configurables:
* `ANI_CLI_PLAYER`: Cambia el reproductor (`mpv`, `vlc`, `iina`).
* `ANI_CLI_QUALITY`: Cambia la calidad (`best`, `1080p`, `720p`, `worst`).
* `ANI_CLI_DOWNLOAD_DIR`: Directorio destino de descargas (por defecto `.`).
* `ANI_CLI_MENU`: Programa de menú (`fzf`, `rofi`, `dmenu`).

---

## 🤝 Créditos y Licencia

Basado originalmente en el proyecto [pystardust/ani-cli](https://github.com/pystardust/ani-cli).
Distribuido bajo licencia [GNU General Public License v3.0 (GPLv3)](LICENSE).
