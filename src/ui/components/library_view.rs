use crate::ui::app::{ActivePane, App};
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table, Widget, Wrap};

pub struct LibraryView;

impl LibraryView {
    pub fn render(app: &mut App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let is_left_active = app.active_pane == ActivePane::List;

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let table_block = Theme::pane_block(
            &format!("BIBLIOTECA OFFLINE ({})", app.library_list.len()),
            is_left_active,
        );

        let rows: Vec<Row> = if app.library_list.is_empty() {
            vec![Row::new(vec![
                Cell::from(""),
                Cell::from(format!(
                    "No se encontraron descargas en: {}",
                    app.config.download_dir.display()
                ))
                .style(Style::default().fg(Theme::GRAY_MID)),
                Cell::from(""),
                Cell::from(""),
            ])]
        } else {
            app.library_list
                .iter()
                .enumerate()
                .map(|(idx, file)| {
                    let num_cell = Cell::from(format!("{:02}", idx + 1));
                    let name_cell = Cell::from(file.file_name.clone());
                    let size_cell = Cell::from(file.size_formatted.clone())
                        .style(Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD));
                    let status_cell = Cell::from("💾 LOCAL")
                        .style(Style::default().fg(Theme::CYAN));

                    Row::new(vec![num_cell, name_cell, size_cell, status_cell])
                })
                .collect()
        };

        let table = Table::new(
            rows,
            [
                Constraint::Length(4),
                Constraint::Min(25),
                Constraint::Length(10),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new(vec!["N°", "ARCHIVO MULTIMEDIA", "TAMAÑO", "ESTADO"])
                .style(Theme::table_header_style()),
        )
        .block(table_block)
        .row_highlight_style(Theme::highlight_row_style())
        .highlight_symbol("▶ ");

        ratatui::widgets::StatefulWidget::render(
            table,
            main_chunks[0],
            buf,
            &mut app.library_table_state,
        );

        // Right details
        let right_block = Theme::pane_block("DETALLES DE ARCHIVO LOCAL", !is_left_active);
        let right_content = if let Some(file) = app.library_list.get(app.library_table_state.selected().unwrap_or(0)) {
            vec![
                Line::from(vec![
                    Span::styled("✦ Archivo:    ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled(&file.file_name, Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("✦ Tamaño:     ", Style::default().fg(Theme::CYAN)),
                    Span::styled(&file.size_formatted, Style::default().fg(Theme::GREEN)),
                ]),
                Line::from(vec![
                    Span::styled("✦ Ubicación:  ", Style::default().fg(Theme::CYAN)),
                    Span::styled(file.path.display().to_string(), Style::default().fg(Theme::GRAY_LIGHT)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("✦ Reproductor:", Style::default().fg(Theme::CYAN)),
                    Span::styled(format!(" {} (Offline)", app.config.player), Style::default().fg(Theme::PURPLE)),
                ]),
                Line::from(""),
                Line::from(Span::styled("─── ACCIONES DISPONIBLES ──────────────────────", Style::default().fg(Theme::GRAY_MID))),
                Line::from(vec![
                    Span::styled("► Presiona ", Style::default().fg(Theme::GRAY_LIGHT)),
                    Span::styled("Enter", Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)),
                    Span::styled(" para iniciar la reproducción inmediatamente sin conexión a internet.", Style::default().fg(Theme::GRAY_LIGHT)),
                ]),
            ]
        } else {
            vec![Line::from(Span::styled(
                "Directorio de descargas vacío.",
                Style::default().fg(Theme::GRAY_MID),
            ))]
        };

        Paragraph::new(right_content)
            .block(right_block)
            .wrap(Wrap { trim: true })
            .render(main_chunks[1], buf);
    }
}
