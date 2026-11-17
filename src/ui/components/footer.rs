use crate::ui::app::App;
use crate::ui::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

pub struct Footer;

impl Footer {
    pub fn render(app: &App, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let has_toast = app.toast.as_ref().is_some_and(|t| {
            t.timestamp.elapsed().as_secs() < 6
        });

        let mut spans = Vec::new();

        if has_toast {
            if let Some(toast) = &app.toast {
                let icon = if toast.is_error { " ✖ " } else { " ✔ " };
                let color = if toast.is_error { Theme::RED } else { Theme::GREEN };
                spans.push(Span::styled(icon, Style::default().fg(color).add_modifier(Modifier::BOLD)));
                spans.push(Span::styled(&toast.message, Style::default().fg(color).add_modifier(Modifier::BOLD)));
                spans.push(Span::styled("   │   ", Style::default().fg(Theme::GRAY_MID)));
            }
        }

        // Key shortcuts
        spans.extend(Theme::key_badge("1-6", "Pestañas"));
        spans.extend(Theme::key_badge("Tab", "Cambiar Panel"));
        spans.extend(Theme::key_badge("j/k", "Navegar"));
        if app.active_tab == crate::ui::app::ActiveTab::Genres && app.genres_column == crate::ui::app::GenresColumn::Animes {
            spans.extend(Theme::key_badge("h/Esc", "Categorías"));
        }
        spans.extend(Theme::key_badge("Enter", "Acción"));
        spans.extend(Theme::key_badge("d", "Descargar"));
        spans.extend(Theme::key_badge("/", "Buscar"));
        spans.extend(Theme::key_badge("q", "Salir"));

        let p = Paragraph::new(Line::from(spans))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Theme::GRAY_MID)),
            )
            .alignment(ratatui::layout::Alignment::Center);

        p.render(area, buf);
    }
}
