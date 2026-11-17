use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Table, Widget};

pub struct HelpView;

impl HelpView {
    pub fn render(app: &App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // 1. Keyboard Shortcuts Table
        let key_rows = vec![
            Row::new(vec![Cell::from("1 - 6").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)), Cell::from("Cambiar pestaña global"), Cell::from("Buscar, Estrenos, Géneros, etc.")]),
            Row::new(vec![Cell::from("Tab / Shift+Tab").style(Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)), Cell::from("Alternar foco de panel"), Cell::from("Cambia entre Lista izquierda y Ficha derecha")]),
            Row::new(vec![Cell::from("j / k  o  ▲ / ▼").style(Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)), Cell::from("Navegación vertical"), Cell::from("Mover cursor por tablas y listas")]),
            Row::new(vec![Cell::from("h / l  o  ◄ / ►").style(Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)), Cell::from("Navegación horizontal"), Cell::from("Cambiar sub-pestañas de inspector")]),
            Row::new(vec![Cell::from("/ (Barra)").style(Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)), Cell::from("Activar buscador"), Cell::from("Permite escribir término de búsqueda")]),
            Row::new(vec![Cell::from("Enter").style(Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)), Cell::from("Acción principal"), Cell::from("Cargar anime / abrir menú de reproducción")]),
            Row::new(vec![Cell::from("d").style(Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)), Cell::from("Descarga directa"), Cell::from("Descargar episodio en disco con ffmpeg")]),
            Row::new(vec![Cell::from("D (Mayús)").style(Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)), Cell::from("Limpiar historial"), Cell::from("Abre confirmación para borrar historial")]),
            Row::new(vec![Cell::from("Esc").style(Style::default().fg(Theme::GRAY_LIGHT).add_modifier(Modifier::BOLD)), Cell::from("Cancelar / Salir modo"), Cell::from("Cierra diálogos o sale del modo edición")]),
            Row::new(vec![Cell::from("q / Ctrl+C").style(Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)), Cell::from("Cerrar aplicación"), Cell::from("Finaliza la sesión de ani-cli-es")]),
        ];

        let keys_table = Table::new(
            key_rows,
            [Constraint::Length(18), Constraint::Length(25), Constraint::Min(25)],
        )
        .header(Row::new(vec!["TECLA / ATAJO", "FUNCIÓN", "DESCRIPCIÓN"]).style(Theme::table_header_style()))
        .block(Theme::pane_block("MAPA DE TECLADO Y ATAJOS", true));

        keys_table.render(chunks[0], buf);

        // 2. System Configuration & Environment
        let config_rows = vec![
            Row::new(vec![Cell::from("Motor TUI"), Cell::from("Ratatui 0.29 + Crossterm 0.28")]),
            Row::new(vec![Cell::from("Runtime Async"), Cell::from("Tokio 1.40 (Full multi-threaded)")]),
            Row::new(vec![Cell::from("Cliente HTTP"), Cell::from("Reqwest (Rustls TLS + Cookies)")]),
            Row::new(vec![Cell::from("Fuente Online"), Cell::from("JKAnime.net (Nativo)")],),
            Row::new(vec![Cell::from("Reproductor"), Cell::from(app.config.player.clone()).style(Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD))]),
            Row::new(vec![Cell::from("Calidad Stream"), Cell::from(app.config.quality.clone()).style(Style::default().fg(Theme::YELLOW))]),
            Row::new(vec![Cell::from("Descargas Dir"), Cell::from(app.config.download_dir.display().to_string())]),
            Row::new(vec![Cell::from("Historial State"), Cell::from(app.config.history_file.display().to_string())]),
            Row::new(vec![Cell::from("Ani-Skip Intro"), Cell::from(if app.config.skip_intro { "Activado" } else { "Desactivado" })]),
            Row::new(vec![Cell::from("Versión"), Cell::from("v5.2.0-es")]),
        ];

        let config_table = Table::new(
            config_rows,
            [Constraint::Length(16), Constraint::Min(20)],
        )
        .header(Row::new(vec!["PARÁMETRO", "VALOR ACTUAL"]).style(Theme::table_header_style()))
        .block(Theme::pane_block("ESTADO DEL ENTORNO", false));

        config_table.render(chunks[1], buf);
    }
}
