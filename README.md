<h1 align="center">
	ani-cli-es 🇲🇽 🇪🇸
</h1>

<h3 align="center">
Una herramienta CLI para buscar y ver anime subtitulado en español desde tu terminal.
</h3>

<p align="center">
  <a href="#instalación"><img src="https://img.shields.io/badge/instalación-rápida-brightgreen"></a>
  <a href="#uso"><img src="https://img.shields.io/badge/uso-fácil-blue"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/licencia-GPLv3-red"></a>
</p>

---

## 📌 Características

- 🇲🇽 🇪🇸 **Soporte Nativo en Español:** Búsqueda y transmisión de anime con subtítulos en español de forma predeterminada.
- ⚡ **Extracción Rápida:** Obtención instantánea de streams HLS (`.m3u8`) directos.
- 🍿 **Soporte para Múltiples Reproductores:** Funciona nativamente con `mpv`, `vlc`, `iina` y descarga con `yt-dlp` / `ffmpeg`.
- 🔁 **Fallback Inteligente:** Si un anime no está en fuentes en español, conmuta automáticamente a fuentes originales con soporte para subtítulos externos.

---

## 🚀 Instalación

### Requisitos previos
Asegúrate de tener instalados en tu sistema:
* `curl`
* `grep`
* `sed`
* `fzf` (o `rofi` / `dmenu`)
* `mpv` (o `vlc` / `iina`)

### Desde el código fuente (Cualquier sistema Linux / macOS)

```sh
# 1. Clonar el repositorio
git clone https://github.com/gerickt/ani-cli-es.git

# 2. Copiar el ejecutable a /usr/local/bin
cd ani-cli-es
sudo cp ani-cli-es /usr/local/bin/
sudo cp ani-cli-es.1 /usr/local/share/man/man1/ 2>/dev/null || true

# 3. Limpiar carpeta temporal
cd .. && rm -rf ani-cli-es
```

---

## 🎮 Uso

### Búsqueda Interactiva
Simplemente escribe `ani-cli-es` seguido del nombre del anime:

```sh
ani-cli-es "one piece"
```

### Opciones Comunes

| Comando | Descripción |
| :--- | :--- |
| `ani-cli-es "anime"` | Búsqueda e interfaz interactiva con `fzf` |
| `ani-cli-es -e 1 "cyberpunk edgerunners"` | Reproduce directamente el episodio 1 |
| `ani-cli-es -e "1-5" "naruto"` | Reproduce en maratón los episodios 1 al 5 |
| `ani-cli-es -d -e 1 "blue lock"` | Descarga el episodio en lugar de reproducirlo |
| `ani-cli-es -c` | Continúa viendo desde tu historial de reproducción |
| `ani-cli-es --vlc "dragon ball"` | Utiliza VLC como reproductor en lugar de MPV |
| `ani-cli-es -L eng "anime"` | Fuerza la búsqueda en inglés |

---

## 🔄 Historial y Configuración

El historial de reproducción se guarda automáticamente en:
`~/.local/state/ani-cli-es/ani-hsts`

Puedes usar las siguientes variables de entorno:
* `ANI_CLI_PLAYER`: Cambia el reproductor por defecto (`mpv`, `vlc`, `iina`).
* `ANI_CLI_QUALITY`: Cambia la calidad preferida (`best`, `1080p`, `720p`, `worst`).
* `ANI_CLI_LANG`: Cambia el idioma preferido (`spa`, `jpn`, `eng`).

---

## 🤝 Créditos y Licencia

Este proyecto es un fork enfocado en la comunidad hispanohablante basado en el trabajo original de [pystardust/ani-cli](https://github.com/pystardust/ani-cli).

Licencia bajo la versión 3 de la [GNU General Public License (GPLv3)](LICENSE).
