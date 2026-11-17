use crate::ui::app::{ActiveTab, App};
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Tabs, Widget};

pub struct Header;

impl Header {
    pub fn render(app: &App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(28), // Logo Badge
                Constraint::Min(40),    // Global Tabs
                Constraint::Length(20), // Status Indicator
            ])
            .split(area);

        // 1. Logo Badge
        let logo_line = Line::from(vec![
            Span::styled(" ✦ ", Style::default().fg(Theme::RED).add_modifier(Modifier::BOLD)),
            Span::styled("ANI-CLI-ES", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" • ", Style::default().fg(Theme::GRAY_MID)),
            Span::styled("v5.2.0-es ", Style::default().fg(Theme::GREEN)),
        ]);
        let logo_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Theme::GRAY_MID));
        Paragraph::new(logo_line)
            .block(logo_block)
            .render(chunks[0], buf);

        // 2. Navigation Tabs
        let titles: Vec<Line> = ActiveTab::all()
            .iter()
            .map(|t| {
                let is_active = *t == app.active_tab;
                let num = (*t as usize) + 1;
                let text = t.title();
                if is_active {
                    Line::from(vec![
                        Span::styled(format!(" [{}] ", num), Style::default().fg(Theme::YELLOW).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{} ", text), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(format!(" {} ", num), Style::default().fg(Theme::GRAY_MID)),
                        Span::styled(format!("{} ", text), Style::default().fg(Theme::GRAY_LIGHT)),
                    ])
                }
            })
            .collect();

        let tabs = Tabs::new(titles)
            .select(app.active_tab as usize)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Theme::BLUE)),
            )
            .highlight_style(
                Style::default()
                    .bg(Theme::BG_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(Span::styled("│", Style::default().fg(Theme::GRAY_MID)));
        tabs.render(chunks[1], buf);

        // 3. Status Indicator
        let (status_text, status_color) = if app.is_loading || app.is_searching {
            let msg = if !app.loading_message.is_empty() {
                format!(" {} {}", app.spinner_char(), app.loading_message)
            } else {
                format!(" {} CARGANDO...", app.spinner_char())
            };
            (msg, Theme::YELLOW)
        } else {
            (" ● EN LÍNEA".to_string(), Theme::GREEN)
        };

        let status_line = Line::from(vec![
            Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]);

        let status_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Theme::GRAY_MID));
        Paragraph::new(status_line)
            .block(status_block)
            .alignment(ratatui::layout::Alignment::Center)
            .render(chunks[2], buf);
    }
}
