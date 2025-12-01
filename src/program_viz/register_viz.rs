use ratatui::{
    layout::{Constraint, Layout},
    symbols::{border, line},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::program::Program;

pub struct RegisterVisualizer<'a> {
    program: &'a Program,
}

impl<'a> Widget for &RegisterVisualizer<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        const NUM_REGS: usize = 5;

        let area_wrapper = Layout::vertical([Constraint::Length(3)]).split(area)[0];
        let cols = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ])
        .split(area_wrapper);

        let vals: [u8; NUM_REGS] = [
            self.program.reg_a().get(),
            self.program.reg_x().get(),
            self.program.reg_y().get(),
            self.program.reg_sp().get(),
            self.program.reg_pc().get(),
        ];
        let titles: [&'static str; NUM_REGS] = ["A", "X", "Y", "SP", "PC"];

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
            Paragraph::new(format!("{:02x}", vals[i]))
                .centered()
                .block(block)
                .render(cols[i], buf);
        }

        // Paragraph::new(self.program.reg_a().get().to_string()).render(area, buf);
    }
}

impl<'a> RegisterVisualizer<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self { program }
    }
}
