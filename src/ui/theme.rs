use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders};

pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // TokyoNight Cyber-Brutalist Palette
    pub const CYAN: Color = Color::Rgb(125, 207, 255);
    pub const BLUE: Color = Color::Rgb(122, 162, 247);
    pub const PURPLE: Color = Color::Rgb(187, 154, 247);
    pub const GREEN: Color = Color::Rgb(158, 206, 106);
    pub const YELLOW: Color = Color::Rgb(224, 175, 104);
    pub const ORANGE: Color = Color::Rgb(255, 158, 100);
    pub const RED: Color = Color::Rgb(247, 118, 142);
    pub const GRAY_DARK: Color = Color::Rgb(41, 46, 66);
    pub const GRAY_MID: Color = Color::Rgb(86, 95, 137);
    pub const GRAY_LIGHT: Color = Color::Rgb(169, 177, 214);
    pub const BG_CARD: Color = Color::Rgb(26, 27, 38);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(36, 40, 59);
    pub const BG_ACTIVE_ROW: Color = Color::Rgb(45, 52, 84);
    pub const TEXT_FG: Color = Color::Rgb(192, 202, 245);

    // Focused active pane border vs subtle inactive border
    pub fn pane_block(title: &str, is_active: bool) -> Block<'static> {
        let (border_color, title_color, border_type) = if is_active {
            (Self::CYAN, Self::YELLOW, BorderType::Double)
        } else {
            (Self::GRAY_MID, Self::GRAY_LIGHT, BorderType::Rounded)
        };

        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(border_color))
            .title(format!(" {} ", title))
            .title_style(
                Style::default()
                    .fg(title_color)
                    .add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() }),
            )
    }

    pub fn modal_block(title: &str) -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Self::PURPLE).add_modifier(Modifier::BOLD))
            .title(format!(" ✦ {} ✦ ", title))
            .title_style(
                Style::default()
                    .fg(Self::YELLOW)
                    .add_modifier(Modifier::BOLD),
            )
    }

    pub fn table_header_style() -> Style {
        Style::default()
            .bg(Self::BG_HIGHLIGHT)
            .fg(Self::CYAN)
            .add_modifier(Modifier::BOLD)
    }

    pub fn highlight_row_style() -> Style {
        Style::default()
            .bg(Self::BG_ACTIVE_ROW)
            .fg(Self::YELLOW)
            .add_modifier(Modifier::BOLD)
    }

    pub fn key_badge(key: &str, label: &str) -> Vec<Span<'static>> {
        vec![
            Span::styled(
                format!(" {} ", key),
                Style::default()
                    .bg(Self::GRAY_DARK)
                    .fg(Self::CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", label),
                Style::default().fg(Self::GRAY_LIGHT),
            ),
        ]
    }
}
