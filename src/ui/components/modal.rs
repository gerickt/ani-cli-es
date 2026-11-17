use crate::ui::app::{App, ModalDialog};
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Widget};

pub struct Modal;

impl Modal {
    pub fn render(app: &App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        match &app.modal {
            ModalDialog::None => {}
            ModalDialog::EpisodeActions { title, ep_no, selected_index, .. } => {
                let popup_area = centered_rect(60, 40, area);
                Clear.render(popup_area, buf);

                let block = Theme::modal_block("ACCIONES DE REPRODUCCIÓN");
                let inner = block.inner(popup_area);
                block.render(popup_area, buf);

                let options = [
                    ("[1]", "▶  Reproducir Episodio (Stream 1080p)", "Lanzar mpv / vlc con mejor enlace"),
                    ("[2]", "⬇️  Descargar Episodio", "Guardar archivo local en disco con ffmpeg"),
                    ("[Esc]", "✖  Cancelar", "Cerrar este diálogo"),
                ];

                let mut lines = Vec::new();
                lines.push(Line::from(vec![
                    Span::styled("✦ Anime:    ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled(title, Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("✦ Episodio: ", Style::default().fg(Theme::CYAN)),
                    Span::styled(format!("Episodio {:02}", ep_no), Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("─── SELECCIONA UNA ACCIÓN ──────────────────────", Style::default().fg(Theme::GRAY_MID))));

                for (idx, (key, label, desc)) in options.iter().enumerate() {
                    let is_sel = idx == *selected_index;
                    let style = if is_sel {
                        Style::default().bg(Theme::BG_ACTIVE_ROW).fg(Theme::YELLOW).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Theme::TEXT_FG)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(format!(" {} ", key), Style::default().bg(Theme::GRAY_DARK).fg(Theme::CYAN)),
                        Span::raw(" "),
                        Span::styled(*label, style),
                        Span::styled(format!("  ({})", desc), Style::default().fg(Theme::GRAY_MID)),
                    ]));
                }

                let p = Paragraph::new(lines);
                p.render(inner, buf);
            }
            ModalDialog::SelectServer {
                title,
                ep_no,
                sources,
                selected_index,
                is_download,
                ..
            } => {
                let popup_area = centered_rect(68, 60, area);
                Clear.render(popup_area, buf);

                let header_title = if *is_download {
                    format!("DESCARGAR EPISODIO {:02} - SELECCIÓN DE SERVIDOR", ep_no)
                } else {
                    format!("REPRODUCIR EPISODIO {:02} - SELECCIÓN DE SERVIDOR", ep_no)
                };
                let block = Theme::modal_block(&header_title);
                let inner = block.inner(popup_area);
                block.render(popup_area, buf);

                let mut lines = Vec::new();
                lines.push(Line::from(vec![
                    Span::styled("✦ Anime:     ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled(title, Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("✦ Episodio:  ", Style::default().fg(Theme::CYAN)),
                    Span::styled(format!("Episodio {:02}", ep_no), Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)),
                    Span::raw("   "),
                    Span::styled("✦ Servidores: ", Style::default().fg(Theme::CYAN)),
                    Span::styled(format!("{} disponibles", sources.len()), Style::default().fg(Theme::ORANGE).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "─── SERVIDORES DISPONIBLES ───────────────────────────────────────────",
                    Style::default().fg(Theme::GRAY_MID),
                )));

                for (idx, source) in sources.iter().enumerate() {
                    let is_sel = idx == *selected_index;
                    let prefix = if is_sel { "▶ " } else { "  " };

                    let (badge, badge_color) = if source.server_name.contains("Mp4upload") {
                        (" (Directo - Fallback)", Theme::CYAN)
                    } else if source.server_name.contains("Principal") || source.server_name.contains("Servidor V") {
                        (" (HLS Directo)", Theme::GREEN)
                    } else {
                        ("", Theme::GRAY_MID)
                    };

                    let row_style = if is_sel {
                        Style::default().bg(Theme::BG_ACTIVE_ROW).fg(Theme::YELLOW).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Theme::TEXT_FG)
                    };

                    let key_tag = format!("[{}]", idx + 1);

                    lines.push(Line::from(vec![
                        Span::styled(prefix, if is_sel { Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD) } else { Style::default() }),
                        Span::styled(format!(" {} ", key_tag), Style::default().bg(Theme::GRAY_DARK).fg(Theme::CYAN)),
                        Span::raw(" "),
                        Span::styled(&source.server_name, row_style),
                        Span::styled(format!(" [{}]", source.quality), Style::default().fg(Theme::PURPLE)),
                        Span::styled(badge, Style::default().fg(badge_color).add_modifier(if is_sel { Modifier::BOLD } else { Modifier::empty() })),
                    ]));
                }

                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "↑/↓ / j/k: Navegar | 1-9: Selección directa | Enter: Confirmar | Esc: Cancelar",
                    Style::default().fg(Theme::GRAY_LIGHT),
                )));

                let p = Paragraph::new(lines);
                p.render(inner, buf);
            }
            ModalDialog::ConfirmClearHistory => {
                let popup_area = centered_rect(50, 30, area);
                Clear.render(popup_area, buf);

                let block = Theme::modal_block("BORRAR HISTORIAL");
                let inner = block.inner(popup_area);
                block.render(popup_area, buf);

                let lines = vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(" ¿Estás seguro de borrar todo el historial? ", Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(Span::styled(" Esta acción eliminará el registro de visualización.", Style::default().fg(Theme::GRAY_LIGHT))),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(" [Enter] ", Style::default().bg(Theme::RED).fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
                        Span::raw(" Confirmar borrado     "),
                        Span::styled(" [Esc] ", Style::default().bg(Theme::GRAY_DARK).fg(Theme::CYAN)),
                        Span::raw(" Cancelar"),
                    ]),
                ];

                let p = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
                p.render(inner, buf);
            }
            ModalDialog::PlayerStatus {
                title,
                ep_no,
                server_name,
                logs,
                is_running,
                has_error,
            } => {
                let popup_area = centered_rect(72, 70, area);
                Clear.render(popup_area, buf);

                let header_title = format!("REPRODUCTOR EN VIVO [mpv] - EPISODIO {:02}", ep_no);
                let block = Theme::modal_block(&header_title);
                let inner = block.inner(popup_area);
                block.render(popup_area, buf);

                let (status_text, status_style) = if *has_error {
                    ("✖ Error al cargar reproducción", Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD))
                } else if *is_running {
                    ("● Reproduciendo en ventana externa...", Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD))
                } else {
                    ("✔ Finalizado", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))
                };

                let mut lines = Vec::new();
                lines.push(Line::from(vec![
                    Span::styled("✦ Anime:     ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled(title, Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("✦ Episodio:  ", Style::default().fg(Theme::CYAN)),
                    Span::styled(format!("Episodio {:02}", ep_no), Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD)),
                    Span::raw("   "),
                    Span::styled("✦ Servidor: ", Style::default().fg(Theme::CYAN)),
                    Span::styled(server_name, Style::default().fg(Theme::ORANGE).add_modifier(Modifier::BOLD)),
                    Span::raw("   "),
                    Span::styled("✦ Estado: ", Style::default().fg(Theme::CYAN)),
                    Span::styled(status_text, status_style),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "─── SALIDA DEL REPRODUCTOR (LOGS) ────────────────────────────────────",
                    Style::default().fg(Theme::GRAY_MID),
                )));

                if logs.is_empty() {
                    lines.push(Line::from(Span::styled("  Esperando logs del reproductor...", Style::default().fg(Theme::GRAY_MID))));
                } else {
                    let available_log_lines = (inner.height.saturating_sub(7) as usize).clamp(5, 15);
                    let start = logs.len().saturating_sub(available_log_lines);
                    for log_line in &logs[start..] {
                        let log_style = if log_line.contains("ERROR") || log_line.contains("Error") || log_line.contains("Failed") {
                            Style::default().fg(Theme::RED)
                        } else if log_line.contains("WARN") || log_line.contains("warning") {
                            Style::default().fg(Theme::YELLOW)
                        } else {
                            Style::default().fg(Theme::GRAY_LIGHT)
                        };
                        lines.push(Line::from(Span::styled(format!("  {}", log_line), log_style)));
                    }
                }

                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    " [q / Esc]: Detener reproductor y cerrar | [Tab]: Minimizar diálogo",
                    Style::default().fg(Theme::GRAY_LIGHT),
                )));

                let p = Paragraph::new(lines);
                p.render(inner, buf);
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
