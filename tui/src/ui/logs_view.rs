use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::text::Line,
    text::Span,
    widgets::{Paragraph, Widget},
};

use emulator::Emulator;

pub fn logs_view(program: &Emulator, area: Rect, buf: &mut Buffer) {
    let lines: Vec<Line> = program
        .get_debug_logs()
        .iter()
        .map(|s| Line::from(Span::raw(s.as_str())))
        .collect();
    Paragraph::new(lines).render(area, buf);
}
