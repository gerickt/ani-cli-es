use crate::ui::app::{ActivePane, App, GenresColumn};
use crate::ui::components::detail_deck::DetailDeck;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};

pub struct GenresView;

impl GenresView {
    pub fn render(app: &mut App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let is_left_active = app.active_pane == ActivePane::List;
        let is_cats_focused = is_left_active && app.genres_column == GenresColumn::Categories;
        let is_animes_focused = is_left_active && app.genres_column == GenresColumn::Animes;

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(26), // Genres List
                Constraint::Percentage(38), // Category Anime Table
                Constraint::Percentage(36), // Detail Inspector
            ])
            .split(area);

        // 1. Genres Selector Table
        let genre_block = Theme::pane_block("GÉNEROS (23)", is_cats_focused);
        let genre_rows: Vec<Row> = app
            .genres_list
            .iter()
            .map(|g| {
                let icon_cell = Cell::from(g.icon.clone());
                let name_cell = Cell::from(g.name.clone());
                Row::new(vec![icon_cell, name_cell])
            })
            .collect();

        let genre_table = Table::new(
            genre_rows,
            [Constraint::Length(4), Constraint::Min(12)],
        )
        .header(Row::new(vec!["", "CATEGORÍA"]).style(Theme::table_header_style()))
        .block(genre_block)
        .row_highlight_style(Theme::highlight_row_style())
        .highlight_symbol("▶ ");

        ratatui::widgets::StatefulWidget::render(
            genre_table,
            main_chunks[0],
            buf,
            &mut app.genres_table_state,
        );

        // 2. Anime in Category Table
        let cat_name = app
            .genres_list
            .get(app.genres_table_state.selected().unwrap_or(0))
            .map(|g| g.name.as_str())
            .unwrap_or("CATÁLOGO");

        let anime_block = Theme::pane_block(
            &format!("CATÁLOGO: {} ({})", cat_name, app.genre_animes.len()),
            is_animes_focused,
        );

        let loading_msg = format!("{} Cargando catálogo...", app.spinner_char());
        let anime_rows: Vec<Row> = if app.genre_animes.is_empty() {
            let empty_text = if app.is_loading {
                loading_msg.as_str()
            } else if is_animes_focused {
                "Sin animes. Presiona h o Esc para volver a géneros."
            } else {
                "Presiona Enter en un género para cargar títulos."
            };
            vec![Row::new(vec![
                Cell::from(""),
                Cell::from(empty_text)
                .style(if app.is_loading {
                    Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::GRAY_MID)
                }),
            ])]
        } else {
            app.genre_animes
                .iter()
                .enumerate()
                .map(|(idx, a)| {
                    let num_cell = Cell::from(format!("{:02}", idx + 1));
                    let title_cell = Cell::from(a.title.clone());
                    Row::new(vec![num_cell, title_cell])
                })
                .collect()
        };

        let anime_table = Table::new(
            anime_rows,
            [Constraint::Length(4), Constraint::Min(20)],
        )
        .header(Row::new(vec!["N°", "TÍTULO DEL ANIME"]).style(Theme::table_header_style()))
        .block(anime_block)
        .row_highlight_style(Theme::highlight_row_style())
        .highlight_symbol("▶ ");

        ratatui::widgets::StatefulWidget::render(
            anime_table,
            main_chunks[1],
            buf,
            &mut app.genre_animes_table_state,
        );

        // 3. DetailDeck Inspector
        DetailDeck::render(app, main_chunks[2], buf);
    }
}
