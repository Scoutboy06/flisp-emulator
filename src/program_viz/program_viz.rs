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
        memory_viz::{self, memory_viz},
        register_viz::register_viz,
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
        let [memory_area, register_area] =
            Layout::horizontal([Constraint::Length(45), Constraint::Min(1)]).areas(area);

        memory_viz(self.program, memory_area, buf);
        register_viz(self.program, register_area, buf);
    }
}
