use crate::ui::app::{ActivePane, App};
use crate::ui::components::detail_deck::DetailDeck;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
    Widget,
};

pub struct SearchView;

impl SearchView {
    pub fn render(app: &mut App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let is_left_active = app.active_pane == ActivePane::List;

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
            .split(area);

        // Left Pane: Search Input + Table
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input box
                Constraint::Min(8),    // Results Table
            ])
            .split(main_chunks[0]);

        // 1. Search Box
        let cursor_char = if app.is_search_editing { "█" } else { "" };
        let input_content = format!("► {}{}", app.search_input, cursor_char);

        let input_border_color = if app.is_search_editing {
            Theme::YELLOW
        } else if is_left_active {
            Theme::CYAN
        } else {
            Theme::GRAY_MID
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(if app.is_search_editing {
                BorderType::Double
            } else {
                BorderType::Rounded
            })
            .border_style(Style::default().fg(input_border_color))
            .title(" 🔍 BÚSQUEDA INTERACTIVA (Presiona '/' para escribir) ")
            .title_style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD));

        Paragraph::new(input_content)
            .style(if app.is_search_editing {
                Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::TEXT_FG)
            })
            .block(input_block)
            .render(left_chunks[0], buf);

        // 2. Table of Results
        let title_block = if app.is_loading {
            format!("CATÁLOGO DE RESULTADOS [{} BUSCANDO...]", app.spinner_char())
        } else {
            format!("CATÁLOGO DE RESULTADOS ({})", app.search_results.len())
        };
        let table_block = Theme::pane_block(
            &title_block,
            is_left_active && !app.is_search_editing,
        );

        let loading_msg = format!("{} Consultando catálogo en JKAnime...", app.spinner_char());
        let rows: Vec<Row> = if app.search_results.is_empty() {
            vec![Row::new(vec![
                Cell::from(""),
                Cell::from(if app.is_loading {
                    loading_msg.as_str()
                } else if app.search_input.is_empty() {
                    "Escribe un título y presiona Enter para buscar anime."
                } else {
                    "No se encontraron títulos con este criterio."
                })
                .style(if app.is_loading {
                    Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::GRAY_MID)
                }),
                Cell::from(""),
                Cell::from(""),
            ])]
        } else {
            app.search_results
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    let num_cell = Cell::from(format!("{:02}", idx + 1));
                    let title_cell = Cell::from(item.title.clone());
                    let type_cell = Cell::from(
                        item.anime_type
                            .as_deref()
                            .unwrap_or("Serie")
                            .to_string(),
                    )
                    .style(Style::default().fg(Theme::PURPLE));

                    let audio_cell = if item.has_latino {
                        Cell::from("🇲🇽 LATINO").style(Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD))
                    } else {
                        Cell::from("🇯🇵 SUB").style(Style::default().fg(Theme::CYAN))
                    };

                    Row::new(vec![num_cell, title_cell, type_cell, audio_cell])
                })
                .collect()
        };

        let table = Table::new(
            rows,
            [
                Constraint::Length(4),
                Constraint::Min(25),
                Constraint::Length(10),
                Constraint::Length(12),
            ],
        )
        .header(
            Row::new(vec!["N°", "TÍTULO", "FORMATO", "AUDIO"])
                .style(Theme::table_header_style()),
        )
        .block(table_block)
        .row_highlight_style(Theme::highlight_row_style())
        .highlight_symbol("▶ ");

        ratatui::widgets::StatefulWidget::render(
            table,
            left_chunks[1],
            buf,
            &mut app.search_table_state,
        );

        // Scrollbar on left table
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        ratatui::widgets::StatefulWidget::render(
            scrollbar,
            left_chunks[1],
            buf,
            &mut app.search_scrollbar_state,
        );

        // Right Pane: DetailDeck Inspector
        DetailDeck::render(app, main_chunks[1], buf);
    }
}
