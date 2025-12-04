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
    program::Program,
    program_viz::{
        clk_cycles_viz::clk_cycles_viz, debug_viz::debug_viz, flags_viz::flags_viz,
        memory_viz::memory_viz, register_viz::register_viz,
    },
};

pub struct ProgramVisualizer<'a> {
    program: &'a mut Program,
    exit: bool,
    is_running: bool,
}

impl<'a> ProgramVisualizer<'a> {
    pub fn viz(program: &'a mut Program) -> io::Result<()> {
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
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('s') => self.program.step(),
            KeyCode::Char('r') => self.program.reset(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl<'a> Widget for &ProgramVisualizer<'a> {
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

        memory_viz(self.program, col0, buf);
        register_viz(self.program, col1, buf);
        flags_viz(self.program, flags_area, buf);
        clk_cycles_viz(self.program, clk_area, buf);
        debug_viz(self.program, col2, buf);
    }
}
