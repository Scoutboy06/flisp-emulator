use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    symbols::{
        border::{self},
        line,
    },
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::io;

use crate::{
    emulator::Emulator,
    event::handle_event,
    ui::{
        clock_cycles_view::clock_cycles_view, flags_view::flags_view, logs_view::logs_view,
        memory_view::memory_view, register_view::register_view,
    },
};

pub struct EmulatorVisualizer<'a> {
    program: &'a mut Emulator,
    exit: bool,
    is_running: bool,
}

impl<'a> EmulatorVisualizer<'a> {
    pub fn viz(program: &'a mut Emulator) -> io::Result<()> {
        let mut visualizer = Self {
            program,
            exit: false,
            is_running: false,
        };
        let mut terminal = ratatui::init();
        let result = visualizer.run(&mut terminal);
        ratatui::restore();
        result
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            handle_event(self.program, event::read()?);
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl<'a> Widget for &EmulatorVisualizer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [col0, col1, col2] = Layout::horizontal([
            Constraint::Length(45),
            Constraint::Length(26),
            Constraint::Min(1),
        ])
        .areas(area);

        let [registers_area, flags_area, clk_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(col1);

        memory_view(self.program, col0, buf);
        register_view(self.program, col1, buf);
        flags_view(self.program, flags_area, buf);
        clock_cycles_view(self.program, clk_area, buf);
        logs_view(self.program, col2, buf);
    }
}
