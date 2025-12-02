use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::Widget,
    symbols::{border, line},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::program::{CCFlag, Program};

pub fn flags_viz(program: &Program, area: Rect, buf: &mut Buffer) {
    const NUM_REGS: usize = 5;

    let area_wrapper = Layout::vertical([Constraint::Length(3)]).split(area)[0];
    let cols = Layout::horizontal([
        Constraint::Length(4),
        Constraint::Length(5),
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(4),
    ])
    .split(area_wrapper);

    let vals: [char; NUM_REGS] = [
        dot(program.reg_cc().get(CCFlag::I)),
        dot(program.reg_cc().get(CCFlag::N)),
        dot(program.reg_cc().get(CCFlag::Z)),
        dot(program.reg_cc().get(CCFlag::V)),
        dot(program.reg_cc().get(CCFlag::C)),
    ];
    let titles: [&'static str; NUM_REGS] = ["I", "N", "Z", "V", "C"];

    let middle_border_set = border::Set {
        top_left: line::ROUNDED.horizontal_down,
        top_right: line::ROUNDED.horizontal_down,
        bottom_left: line::ROUNDED.horizontal_up,
        bottom_right: line::ROUNDED.horizontal_up,
        ..border::ROUNDED
    };
    let blocks = (0..NUM_REGS).map(|i| {
        let borders = match i {
            0 => Borders::LEFT | Borders::TOP | Borders::BOTTOM,
            1 => Borders::ALL,
            _ => Borders::RIGHT | Borders::TOP | Borders::BOTTOM,
        };
        let border_set = match i {
            0 => border::ROUNDED,
            n if n == NUM_REGS - 1 => border::ROUNDED,
            _ => middle_border_set,
        };
        Block::default()
            .borders(borders)
            .border_set(border_set)
            .title(Line::from(titles[i]).centered())
    });

    for (i, block) in blocks.enumerate() {
        Paragraph::new(vals[i].to_string())
            .centered()
            .block(block)
            .render(cols[i], buf);
    }
}

fn dot(b: bool) -> char {
    if b { '●' } else { '○' }
}
