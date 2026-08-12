# Guía de Contribución a ani-cli-es 🇲🇽 🇪🇸

¡Gracias por tu interés en contribuir a **ani-cli-es**! Este proyecto nace con el objetivo de brindar la mejor experiencia CLI para ver anime subtitulado y doblado en español para toda la comunidad hispanohablante.

---

## 🌟 ¡Todas las contribuciones son bienvenidas!

A diferencia del repositorio original, en **ani-cli-es** nos encanta recibir mejoras y nuevas características:

- **Nuevas Fuentes y Scrapers:** Soporte o mejoras para sitios en español (JKAnime, AnimeFLV, MonosChinos, AnimeLatinoHD, etc.).
- **Compatibilidad de Reproductores:** Mejoras para `mpv`, `VLC`, `IINA`, `yt-dlp` y reproductores en Android / iOS / Steam Deck.
- **Corrección de Errores y Refactorización:** Arreglos de scraping, optimización de comandos en Shell y regexes.

---

## 🛠️ Reglas para Pull Requests (PRs)

1. **Estilo de Código:** Ejecuta `shfmt` para mantener la sangría de 4 espacios:
   ```sh
   shfmt -i 4 -ci -w ani-cli-es
   ```
2. **Validación POSIX:** Asegúrate de que el script no rompa compatibilidad con Shell Estándar usando `shellcheck`:
   ```sh
   shellcheck -s sh ani-cli-es
   ```
3. **Documentación:** Si tu cambio añade una nueva opción o bandera, actualiza [`README.md`](./README.md) y [`ani-cli-es.1`](./ani-cli-es.1).
4. **Sin Dependencias Innecesarias:** Priorizamos el uso de herramientas Unix estándar (`curl`, `grep`, `sed`).

---

## 💬 Comunidad y Comunicación

- **Discord:** Actualmente **no contamos con un servidor de Discord**, pero si el interés y la comunidad crecen, ¡abriremos uno oficialmente!
- **Discusiones e Issues:** Por el momento, puedes reportar errores, proponer ideas o hacer preguntas abriendo un **Issue** o **Discussion** en GitHub.

---

## 📢 OTRAS FORMAS DE AYUDAR

- Dale una ⭐ **Star** al repositorio en GitHub para apoyar el proyecto.
- Comparte **ani-cli-es** con tus amigos y la comunidad anime.
- Ayuda a probar nuevos episodios y reportar fuentes caídas.
