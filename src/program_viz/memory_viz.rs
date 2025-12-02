use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols::{border, line},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::program::{Program, Register};

pub fn memory_viz(
    program: &Program,
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
) {
    let area_wrapper = Layout::vertical([Constraint::Length(34)]).split(area)[0];

    let [address_area, memory_read_area, ascii_area] = Layout::horizontal([
        Constraint::Length(5),
        Constraint::Length(29),
        Constraint::Length(11),
    ])
    .areas(area_wrapper);

    render_address_area(program, address_area, buf);
    render_memory_area(program, memory_read_area, buf);
    render_ascii_area(program, ascii_area, buf);
}

fn render_address_area(program: &Program, area: Rect, buf: &mut Buffer) {
    let addresses = (0u8..32)
        .map(|b| format!("{:02x}\n", b * 8))
        .collect::<String>();
    let block = Block::default()
        .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
        .border_set(border::ROUNDED)
        .title(Line::from("Addr"));

    Paragraph::new(addresses)
        .centered()
        .block(block)
        .render(area, buf);
}

fn render_memory_area(program: &Program, area: Rect, buf: &mut Buffer) {
    let mut memory_str = String::with_capacity(24 * 32);
    for row in 0u8..32 {
        for col in 0u8..8 {
            let b = program.memory_at(row * 8 + col);
            memory_str.push_str(&format!("{:02x}", b));
            if col != 7 {
                memory_str.push(' ');
            }
        }
        memory_str.push('\n');
    }

    let border_set = border::Set {
        top_left: line::ROUNDED.horizontal_down,
        top_right: line::ROUNDED.horizontal_down,
        bottom_left: line::ROUNDED.horizontal_up,
        bottom_right: line::ROUNDED.horizontal_up,
        ..border::ROUNDED
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .title(Line::from(" Memory ").centered());

    Paragraph::new(memory_str)
        .centered()
        .block(block)
        .render(area, buf);
}

fn render_ascii_area(program: &Program, area: Rect, buf: &mut Buffer) {
    let mut ascii_str = String::with_capacity(8 * 32);
    for row in 0u8..32 {
        for col in 0u8..8 {
            let b = program.memory_at(row * 8 + col);
            ascii_str.push(visualize_ascii(b));
        }
        ascii_str.push('\n');
    }

    let block = Block::default()
        .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
        .border_set(border::ROUNDED)
        .title(Line::from(" Ascii ").centered());

    Paragraph::new(ascii_str)
        .centered()
        .block(block)
        .render(area, buf);
}

fn visualize_ascii(b: u8) -> char {
    match b {
        0 => '0',
        32..=126 | 128 | 130..=140 | 142 | 145..=156 | 158..=255 => b as char,
        _ => '•',
    }
}
