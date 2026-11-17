use crate::ui::app::{ActivePane, App};
use crate::ui::components::detail_deck::DetailDeck;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Scrollbar, ScrollbarOrientation, Table};

pub struct EstrenosView;

impl EstrenosView {
    pub fn render(app: &mut App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let is_left_active = app.active_pane == ActivePane::List;

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
            .split(area);

        let table_block = Theme::pane_block(
            &format!("CARTELERA DE EMISIONES HOY ({})", app.estrenos_list.len()),
            is_left_active,
        );

        let loading_msg = format!("{} Obteniendo estrenos en vivo desde JKAnime...", app.spinner_char());
        let rows: Vec<Row> = if app.estrenos_list.is_empty() {
            vec![Row::new(vec![
                Cell::from(""),
                Cell::from(if app.is_loading {
                    loading_msg.as_str()
                } else {
                    "No hay emisiones disponibles en este momento."
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
            app.estrenos_list
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    let num_cell = Cell::from(format!("{:02}", idx + 1));
                    let tag_cell = Cell::from("🌟 HOY").style(Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD));
                    let title_cell = Cell::from(item.title.clone());
                    let ep_cell = Cell::from(format!("Ep. {:02}", item.episode))
                        .style(Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD));

                    Row::new(vec![num_cell, tag_cell, title_cell, ep_cell])
                })
                .collect()
        };

        let table = Table::new(
            rows,
            [
                Constraint::Length(4),
                Constraint::Length(8),
                Constraint::Min(25),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new(vec!["N°", "ESTADO", "TÍTULO DEL ANIME", "EMISIÓN"])
                .style(Theme::table_header_style()),
        )
        .block(table_block)
        .row_highlight_style(Theme::highlight_row_style())
        .highlight_symbol("▶ ");

        ratatui::widgets::StatefulWidget::render(
            table,
            main_chunks[0],
            buf,
            &mut app.estrenos_table_state,
        );

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        ratatui::widgets::StatefulWidget::render(
            scrollbar,
            main_chunks[0],
            buf,
            &mut app.estrenos_scrollbar_state,
        );

        DetailDeck::render(app, main_chunks[1], buf);
    }
}
