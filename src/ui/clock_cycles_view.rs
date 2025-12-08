use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::emulator::Emulator;

pub fn clock_cycles_view(program: &Emulator, area: Rect, buf: &mut Buffer) {
    let clk_cycles = program.clk_count();
    let clk_cycles_str = format!(" CLK Cycles: {}", clk_cycles);

    Paragraph::new(clk_cycles_str).render(area, buf);
}
