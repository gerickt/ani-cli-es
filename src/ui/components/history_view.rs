use crate::ui::app::{ActivePane, App};
use crate::ui::components::detail_deck::DetailDeck;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};

pub struct HistoryView;

impl HistoryView {
    pub fn render(app: &mut App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let is_left_active = app.active_pane == ActivePane::List;

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
            .split(area);

        let table_block = Theme::pane_block(
            &format!("HISTORIAL DE REPRODUCCIÓN ({})", app.history_list.len()),
            is_left_active,
        );

        let rows: Vec<Row> = if app.history_list.is_empty() {
            vec![Row::new(vec![
                Cell::from(""),
                Cell::from("No hay historial de reproducción registrado aún.")
                    .style(Style::default().fg(Theme::GRAY_MID)),
                Cell::from(""),
                Cell::from(""),
            ])]
        } else {
            app.history_list
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    let num_cell = Cell::from(format!("{:02}", idx + 1));
                    let title_cell = Cell::from(item.title.clone());
                    let last_ep_cell = Cell::from(format!("Ep. {:02}", item.episode))
                        .style(Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD));
                    let next_ep_cell = Cell::from(format!("▶ Ep. {:02}", item.episode + 1))
                        .style(Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD));

                    Row::new(vec![num_cell, title_cell, last_ep_cell, next_ep_cell])
                })
                .collect()
        };

        let table = Table::new(
            rows,
            [
                Constraint::Length(4),
                Constraint::Min(25),
                Constraint::Length(12),
                Constraint::Length(14),
            ],
        )
        .header(
            Row::new(vec!["N°", "TÍTULO DEL ANIME", "VISTO", "CONTINUAR"])
                .style(Theme::table_header_style()),
        )
        .block(table_block)
        .row_highlight_style(Theme::highlight_row_style())
        .highlight_symbol("▶ ");

        ratatui::widgets::StatefulWidget::render(
            table,
            main_chunks[0],
            buf,
            &mut app.history_table_state,
        );

        DetailDeck::render(app, main_chunks[1], buf);
    }
}
