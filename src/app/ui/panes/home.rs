use crate::app::App;
use ratatui::{
    Frame,
    layout::{Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, Paragraph},
};

pub fn draw_home(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;

    // Apply dim modifier to the entire background
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                let current_style = cell.style();
                cell.set_style(current_style.add_modifier(Modifier::DIM));
            }
        }
    }

    let modal_w = 72u16.min(area.width);
    let modal_h = 32u16.min(area.height);
    let x = area.x + (area.width.saturating_sub(modal_w)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_h)) / 2;
    let modal_area = Rect::new(x, y, modal_w, modal_h);

    f.render_widget(Clear, modal_area);

    let accent = Color::from(theme.ui.normal_mode_bg.clone());
    let dim = Color::from(theme.ui.dim.clone());
    let sel_bg = Color::from(theme.ui.selection_bg.clone());
    let sel_fg = Color::from(theme.ui.selection_fg.clone());
    let normal_fg = theme.ui.foreground.clone().map(Color::from).unwrap_or(Color::White);

    let title_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(dim))
        .title(Span::styled(" [ Fount Home ] ", title_style));
    
    f.render_widget(block, modal_area);

    let inner = modal_area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 2,
    });

    let mut home_lines = Vec::new();

    // ASCII LOGO
    let logo_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
    for row in &[
        "███████╗ ██████╗ ██╗   ██╗███╗   ██╗████████╗",
        "██╔════╝██╔═══██╗██║   ██║████╗  ██║╚══██╔══╝",
        "█████╗  ██║   ██║██║   ██║██╔██╗ ██║   ██║   ",
        "██╔══╝  ██║   ██║██║   ██║██║╚██╗██║   ██║   ",
        "██║     ╚██████╔╝╚██████╔╝██║ ╚████║   ██║   ",
        "╚═╝      ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ",
    ] {
        home_lines.push(Line::from(Span::styled(*row, logo_style)));
    }

    home_lines.push(Line::from(""));
    home_lines.push(Line::from(Span::styled(
        "Write blockbusters in your terminal.",
        Style::default().fg(dim).add_modifier(Modifier::ITALIC),
    )));
    home_lines.push(Line::from(""));
    home_lines.push(Line::from(Span::styled("─".repeat(40), Style::default().fg(dim))));
    home_lines.push(Line::from(""));

    // MAIN MENU
    let menu_options = [
        "New File",
        "Open File",
        "Tutorial",
        "Exit",
    ];

    for (i, label) in menu_options.iter().enumerate() {
        let is_sel = i == app.home_selected;
        let text = if is_sel { format!(" › {} ", label) } else { format!("   {}   ", label) };
        let style = if is_sel {
            Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(normal_fg)
        };
        home_lines.push(Line::from(Span::styled(text, style)));
        home_lines.push(Line::from(""));
    }

    // RECENT DOCUMENTS
    if !app.recent_files.is_empty() {
        home_lines.push(Line::from(""));
        home_lines.push(Line::from(Span::styled("[ Recent Files ]", Style::default().fg(dim).add_modifier(Modifier::BOLD))));
        home_lines.push(Line::from(""));
        
        for (i, path) in app.recent_files.iter().take(4).enumerate() {
            let idx = menu_options.len() + i;
            let is_sel = idx == app.home_selected;
            
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "Unknown".to_string());
            let text = if is_sel { format!(" › {} ", name) } else { format!("   {}   ", name) };
            
            let style = if is_sel {
                Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(dim)
            };
            
            home_lines.push(Line::from(Span::styled(text, style)));
        }
    }

    f.render_widget(Paragraph::new(home_lines).alignment(Alignment::Center), inner);
}
