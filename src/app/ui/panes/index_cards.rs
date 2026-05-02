use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph, Wrap},
};

pub fn draw_index_cards(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let accent = Color::from(theme.ui.normal_mode_bg.clone());
    let _dim = Color::from(theme.ui.dim.clone());
    let normal_fg = theme.ui.foreground.clone().map(Color::from).unwrap_or(Color::White);
    let selection_bg = Color::from(theme.ui.selection_bg.clone());
    let selection_fg = Color::from(theme.ui.selection_fg.clone());
    let bg_color = theme.ui.background.as_ref()
        .map(|c| Color::from(c.clone()))
        .unwrap_or(Color::Reset);

    let cards = app.extract_scene_cards();
    let grid_area = area;

    // Grid details
    let columns = 3;
    let card_w = (grid_area.width.saturating_sub(4)) / columns; // Accounting for gaps
    let card_h = 10;
    let visible_rows = (grid_area.height / card_h) as usize;
    
    // Auto-scrolling logic
    let selected_row = (app.selected_card_idx / columns as usize) as usize;
    if selected_row < app.card_row_offset {
        app.card_row_offset = selected_row;
    } else if selected_row >= app.card_row_offset + visible_rows {
        app.card_row_offset = selected_row.saturating_sub(visible_rows) + 1;
    }

    for (i, card) in cards.iter().enumerate() {
        let card_row = i / columns as usize;
        if card_row < app.card_row_offset || card_row >= app.card_row_offset + visible_rows {
            continue;
        }
        
        let row_in_view = card_row - app.card_row_offset;
        let col = i % columns as usize;
        
        let x = grid_area.x + 2 + (col as u16 * (card_w + 1));
        let y = grid_area.y + (row_in_view as u16 * card_h);
        
        let card_rect = Rect::new(x, y, card_w, card_h - 1); // -1 for vertical gap
        let is_selected = i == app.selected_card_idx;
        

        // --- DRAW CARD CONTENT ---
        let base_style = Style::default().bg(bg_color);
        
        let mut border_style = theme.secondary_style();
        
        if is_selected {
            border_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
        }
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(border_style)
            .style(base_style);
            
        f.render_widget(block, card_rect);
        
        // Header Bar (ASCII Tab)
        let header_bar_rect = Rect::new(card_rect.x + 1, card_rect.y, card_rect.width - 2, 1);
        let header_style = if is_selected {
            Style::default().fg(accent).add_modifier(Modifier::BOLD)
        } else {
            theme.secondary_style()
        };
        
        let header_label = if let Some(ref num) = card.scene_num {
            format!("SCENE {}", num)
        } else {
            format!("SCENE {}", i + 1)
        };

        let label_style = if let Some(c) = card.color { 
            header_style.fg(c) 
        } else { 
            header_style 
        };
        
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled("[ ", header_style),
            Span::styled(header_label, label_style),
            Span::styled(" ]", header_style),
        ])), header_bar_rect);

        // Content Area
        let inner = Rect::new(card_rect.x + 1, card_rect.y + 2, card_rect.width - 2, card_rect.height - 3);
        
        let mut card_lines = Vec::new();
        
        // Heading
        let heading_content = if is_selected && app.is_card_editing && app.is_heading_editing {
            format!("{}|", app.card_input_buffer)
        } else {
            let h = card.heading.trim_start_matches('.').to_string();
            if h.is_empty() { "[No Heading]".to_string() } else { h }
        };
        
        let heading_style = if is_selected && app.is_card_editing && app.is_heading_editing {
            theme.warning_style().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(normal_fg).add_modifier(Modifier::BOLD)
        };
        
        card_lines.push(Line::from(Span::styled(heading_content, if let Some(c) = card.color { heading_style.fg(c) } else { heading_style })));
        card_lines.push(Line::from(Span::styled(" ", Style::default()))); // Spacer
        
        // Synopsis
        let syn_content = if is_selected && app.is_card_editing && !app.is_heading_editing {
            format!("{}|", app.card_input_buffer)
        } else if !card.synopsis.is_empty() {
            card.synopsis.clone()
        } else if !card.preview.is_empty() {
            card.preview.clone()
        } else {
            "Tap Enter to plan...".to_string()
        };
        
        let syn_style = if is_selected && app.is_card_editing && !app.is_heading_editing {
            Style::default().fg(selection_fg).bg(selection_bg)
        } else if !card.synopsis.is_empty() {
            Style::default().fg(normal_fg).add_modifier(Modifier::ITALIC)
        } else {
            theme.secondary_style().add_modifier(Modifier::ITALIC)
        };
        
        card_lines.push(Line::from(Span::styled(syn_content, syn_style)));
        
        let p = Paragraph::new(card_lines)
            .wrap(Wrap { trim: true });
            
        f.render_widget(p, inner);
    }
}
