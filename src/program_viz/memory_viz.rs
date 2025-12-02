use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    symbols::{border, line},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::program::Program;
use crate::register::Register;

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
    let mut lines: Vec<Line> = Vec::with_capacity(32);
    for row in 0..32_u8 {
        let addr = row * 8;
        let s = format!("{:02x}", addr);
        let span = Span::default().content(s).fg(Color::DarkGray);
        lines.push(Line::from(span));
    }

    let block = Block::default()
        .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
        .border_set(border::ROUNDED)
        .title(Line::from("Addr"));

    Paragraph::new(lines)
        .centered()
        .block(block)
        .render(area, buf);
}

fn render_memory_area(program: &Program, area: Rect, buf: &mut Buffer) {
    let pc = program.reg_pc().get();
    let mut lines: Vec<Line> = Vec::with_capacity(32);
    for row in 0..32_u8 {
        let mut line: Vec<Span> = Vec::with_capacity(15);
        for col in 0..8_u8 {
            let b_idx = row * 8 + col;
            let b = program.memory_at(b_idx);
            let s = format!("{:02x}", b);
            let span = if b_idx == pc {
                Span::default().content(s).bg(Color::White).fg(Color::Black)
            } else if b == 0 {
                Span::default().content(s).fg(Color::DarkGray)
            } else {
                Span::raw(s)
            };
            line.push(span);

            if col != 7 {
                line.push(Span::raw(" "))
            }
        }
        lines.push(Line::from(line));
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

    Paragraph::new(lines)
        .centered()
        .block(block)
        .render(area, buf);
}

fn render_ascii_area(program: &Program, area: Rect, buf: &mut Buffer) {
    let pc = program.reg_pc().get();
    let mut lines: Vec<Line> = Vec::with_capacity(32);
    for row in 0..32_u8 {
        let mut line: Vec<Span> = Vec::with_capacity(8);
        for col in 0..8_u8 {
            let b_idx = row * 8 + col;
            let b = program.memory_at(b_idx);
            let s = visualize_ascii(b).to_string();

            let span = if b_idx == pc {
                Span::default().content(s).bg(Color::White).fg(Color::Black)
            } else if b == 0 {
                Span::default().content(s).fg(Color::DarkGray)
            } else {
                Span::raw(s)
            };
            line.push(span);
        }
        lines.push(Line::from(line));
    }

    let block = Block::default()
        .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
        .border_set(border::ROUNDED)
        .title(Line::from(" Ascii ").centered());

    Paragraph::new(lines)
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
