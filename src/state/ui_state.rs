use ratatui::{
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

pub struct UiState<'a> {
    pub mode: InputMode,
    pub selected_memory_addr: u8,
    pub bottom_help: Paragraph<'a>,
}

pub enum InputMode {
    Normal,
    MemoryEditor,
}

impl<'a> Default for UiState<'a> {
    fn default() -> Self {
        let mut s = Self {
            mode: InputMode::Normal,
            selected_memory_addr: 0,
            bottom_help: Paragraph::new(vec![]),
        };
        s.set_state(InputMode::Normal);
        s
    }
}

impl<'a> UiState<'a> {
    pub fn set_state(&mut self, mode: InputMode) {
        self.mode = mode;

        fn line<'b>(key: &'b str, desc: &'b str) -> Line<'b> {
            Line::from(vec![
                Span::default()
                    .content(key)
                    .bg(Color::Green)
                    .fg(Color::White),
                Span::raw(format!(": {}", desc)),
            ])
        }

        self.bottom_help = match self.mode {
            InputMode::Normal => Paragraph::new(vec![
                line("<Space>", "Start/Pause execution"),
                line("<s>", "Step one instruction"),
                // line("<r>", "Open register editor"),
                line("<m>", "Open memory editor"),
                // line("<b>", "Open breakpoint manager"),
                // line("<B>", "Quick toggle breakpoint at current PC"),
                line("<q>", "Quit program"),
            ]),
            InputMode::MemoryEditor => todo!(),
        }
    }
}
