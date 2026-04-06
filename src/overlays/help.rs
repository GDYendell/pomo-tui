use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Padding},
    Frame,
};
use ratatui_input_manager::{widgets::Help, CrosstermBackend, KeyBind};

use super::util::centered_rect;

pub fn render_help_overlay(frame: &mut Frame, keybinds: &[KeyBind<CrosstermBackend>]) {
    let key_col_width = keybinds
        .iter()
        .map(|kb| {
            let lens: Vec<usize> = kb.pressed.iter().map(|p| p.to_string().len()).collect();
            lens.iter().sum::<usize>() + lens.len().saturating_sub(1) * 2
        })
        .max()
        .unwrap_or(0)
        .max(3) as u16;

    let desc_col_width = keybinds
        .iter()
        .map(|kb| kb.description.len())
        .max()
        .unwrap_or(0) as u16;

    // 2 borders + 2 horizontal padding + key column + 1 default table column spacing + description column
    let overlay_width =
        (4 + key_col_width + 1 + desc_col_width).min(frame.area().width.saturating_sub(4));
    let overlay_height = (keybinds.len() as u16 + 2).min(frame.area().height.saturating_sub(4));

    let overlay_area = centered_rect(frame.area(), overlay_width, overlay_height);
    frame.render_widget(Clear, overlay_area);

    let help = Help::new(keybinds)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .padding(Padding::horizontal(1)),
        )
        .key_style(Style::default().fg(Color::Yellow));
    frame.render_widget(help, overlay_area);
}
