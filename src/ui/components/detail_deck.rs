use crate::ui::app::{ActivePane, App, DetailTab};
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, LineGauge, Paragraph, Row, Table, Tabs, Widget, Wrap,
};

pub struct DetailDeck;

impl DetailDeck {
    pub fn render(app: &mut App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let is_pane_active = app.active_pane == ActivePane::Detail;
        let title = if app.is_loading {
            format!("DETALLES Y REPRODUCTOR [{} CARGANDO]", app.spinner_char())
        } else {
            "DETALLES Y REPRODUCTOR".to_string()
        };
        let outer_block = Theme::pane_block(&title, is_pane_active);
        let inner_area = outer_block.inner(area);
        outer_block.render(area, buf);

        if let Some(details) = &app.current_details {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Sub-tabs
                    Constraint::Length(2), // Progress LineGauge
                    Constraint::Min(5),    // Content according to detail_tab
                ])
                .split(inner_area);

            // 1. Detail Tabs
            let tab_titles = vec![
                Line::from(Span::styled(" [1] 📋 Ficha & Sinopsis ", Style::default())),
                Line::from(Span::styled(
                    format!(" [2] 📺 Episodios ({}) ", app.episodes_list.len()),
                    Style::default(),
                )),
                Line::from(Span::styled(
                    format!(" [3] 🔁 Relacionados ({}) ", details.related_seasons.len()),
                    Style::default(),
                )),
            ];

            let tabs = Tabs::new(tab_titles)
                .select(app.detail_tab as usize)
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_style(Style::default().fg(Theme::GRAY_MID)),
                )
                .highlight_style(
                    Style::default()
                        .fg(Theme::YELLOW)
                        .add_modifier(Modifier::BOLD),
                )
                .divider(Span::raw("│"));
            tabs.render(chunks[0], buf);

            // 2. Progress Gauge (Watched vs Total)
            let last_watched_ep = app
                .history_list
                .iter()
                .find(|h| h.slug == details.slug)
                .map(|h| h.episode)
                .unwrap_or(0);

            let total_eps = if details.episodes_total == 0 { 12 } else { details.episodes_total };
            let ratio = (last_watched_ep as f64 / total_eps as f64).clamp(0.0, 1.0);
            let gauge_label = format!("Progreso: Ep. {} / {} ({}%)", last_watched_ep, total_eps, (ratio * 100.0) as u32);

            let gauge = LineGauge::default()
                .block(Block::default())
                .filled_style(
                    Style::default()
                        .fg(if ratio >= 1.0 { Theme::GREEN } else { Theme::CYAN })
                        .bg(Theme::GRAY_DARK),
                )
                .line_set(ratatui::symbols::line::THICK)
                .label(gauge_label)
                .ratio(ratio);
            gauge.render(chunks[1], buf);

            // 3. Tab content
            match app.detail_tab {
                DetailTab::Overview => {
                    let mut lines = Vec::new();

                    lines.push(Line::from(vec![
                        Span::styled("✦ Título:  ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                        Span::styled(&details.title, Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)),
                    ]));

                    let mut badges = vec![
                        Span::styled(format!(" [{}] ", details.anime_type), Style::default().bg(Theme::BG_HIGHLIGHT).fg(Theme::PURPLE)),
                        Span::raw(" "),
                        Span::styled(format!(" [{}] ", details.status), Style::default().bg(Theme::BG_HIGHLIGHT).fg(Theme::GREEN)),
                        Span::raw(" "),
                        Span::styled(format!(" [{}] ", details.duration), Style::default().bg(Theme::BG_HIGHLIGHT).fg(Theme::YELLOW)),
                        Span::raw(" "),
                        Span::styled(format!(" [Año: {}] ", details.year), Style::default().bg(Theme::BG_HIGHLIGHT).fg(Theme::GRAY_LIGHT)),
                    ];

                    let has_latino = details.genres.iter().any(|g| g.to_lowercase().contains("latino"));
                    if has_latino {
                        badges.push(Span::raw(" "));
                        badges.push(Span::styled(" [🇲🇽 LATINO] ", Style::default().bg(Theme::BG_HIGHLIGHT).fg(Theme::RED)));
                    }

                    lines.push(Line::from(badges));
                    lines.push(Line::from(vec![
                        Span::styled("✦ Estudio: ", Style::default().fg(Theme::CYAN)),
                        Span::styled(&details.studio, Style::default().fg(Theme::TEXT_FG)),
                    ]));

                    if !details.genres.is_empty() {
                        let genre_spans: Vec<Span> = details
                            .genres
                            .iter()
                            .map(|g| Span::styled(format!(" #{} ", g), Style::default().fg(Theme::GRAY_LIGHT)))
                            .collect();
                        let mut g_line = vec![Span::styled("✦ Géneros: ", Style::default().fg(Theme::CYAN))];
                        g_line.extend(genre_spans);
                        lines.push(Line::from(g_line));
                    }

                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "─── SINOPSIS OFICIAL ───────────────────────────────────────────",
                        Style::default().fg(Theme::GRAY_MID),
                    )));

                    if !details.synopsis.is_empty() {
                        for s in details.synopsis.lines() {
                            lines.push(Line::from(Span::styled(s, Style::default().fg(Theme::TEXT_FG))));
                        }
                    } else {
                        lines.push(Line::from(Span::styled(
                            "Sinopsis no disponible en los servidores de JKAnime.",
                            Style::default().fg(Theme::GRAY_MID),
                        )));
                    }

                    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
                    p.render(chunks[2], buf);
                }
                DetailTab::Episodes => {
                    let rows: Vec<Row> = app
                        .episodes_list
                        .iter()
                        .map(|ep| {
                            let ep_num_cell = Cell::from(format!("{:02}", ep.number));
                            let title_cell = Cell::from(ep.title.clone());
                            let date_cell = Cell::from(ep.date.as_deref().unwrap_or("N/A"));
                            let badge_cell = if ep.is_new {
                                Cell::from("🆕 NUEVO").style(Style::default().fg(Theme::YELLOW))
                            } else if ep.number <= last_watched_ep {
                                Cell::from("✔ VISTO").style(Style::default().fg(Theme::GREEN))
                            } else {
                                Cell::from("● PENDIENTE").style(Style::default().fg(Theme::GRAY_MID))
                            };

                            Row::new(vec![ep_num_cell, title_cell, date_cell, badge_cell])
                        })
                        .collect();

                    let table = Table::new(
                        rows,
                        [
                            Constraint::Length(4),
                            Constraint::Min(20),
                            Constraint::Length(12),
                            Constraint::Length(12),
                        ],
                    )
                    .header(
                        Row::new(vec!["N°", "TÍTULO", "EMISIÓN", "ESTADO"])
                            .style(Theme::table_header_style()),
                    )
                    .row_highlight_style(Theme::highlight_row_style())
                    .highlight_symbol("▶ ");

                    ratatui::widgets::StatefulWidget::render(
                        table,
                        chunks[2],
                        buf,
                        &mut app.episodes_table_state,
                    );
                }
                DetailTab::Related => {
                    if details.related_seasons.is_empty() {
                        let p = Paragraph::new("No hay temporadas relacionadas o secuelas registradas.")
                            .style(Style::default().fg(Theme::GRAY_MID));
                        p.render(chunks[2], buf);
                    } else {
                        let rows: Vec<Row> = details
                            .related_seasons
                            .iter()
                            .enumerate()
                            .map(|(idx, rel)| {
                                Row::new(vec![
                                    Cell::from(format!("{:02}", idx + 1)),
                                    Cell::from(rel.title.clone()),
                                    Cell::from(rel.slug.clone()).style(Style::default().fg(Theme::GRAY_MID)),
                                ])
                            })
                            .collect();

                        let table = Table::new(
                            rows,
                            [
                                Constraint::Length(4),
                                Constraint::Min(25),
                                Constraint::Length(30),
                            ],
                        )
                        .header(
                            Row::new(vec!["N°", "TEMPORADA / SECUELA", "SLUG"])
                                .style(Theme::table_header_style()),
                        )
                        .row_highlight_style(Theme::highlight_row_style())
                        .highlight_symbol("▶ ");

                        ratatui::widgets::StatefulWidget::render(
                            table,
                            chunks[2],
                            buf,
                            &mut app.related_table_state,
                        );
                    }
                }
            }
        } else if app.is_loading {
            let loading_text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("  {} ", app.spinner_char()), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled("Obteniendo ficha técnica y lista de episodios...", Style::default().fg(Theme::CYAN)),
                ]),
            ];
            Paragraph::new(loading_text).render(inner_area, buf);
        } else {
            let empty_text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  ✦ ", Style::default().fg(Theme::CYAN)),
                    Span::styled("Selecciona un anime de la lista para inspeccionar su ficha técnica.", Style::default().fg(Theme::GRAY_LIGHT)),
                ]),
                Line::from(vec![
                    Span::styled("  ✦ ", Style::default().fg(Theme::CYAN)),
                    Span::styled("Presiona ", Style::default().fg(Theme::GRAY_MID)),
                    Span::styled("Tab", Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)),
                    Span::styled(" para alternar el foco entre paneles.", Style::default().fg(Theme::GRAY_MID)),
                ]),
            ];
            Paragraph::new(empty_text).render(inner_area, buf);
        }
    }
}
