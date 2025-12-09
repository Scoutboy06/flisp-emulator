use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

use crate::ui::EmulatorVisualizer;

pub fn handle_event(ui: &mut EmulatorVisualizer, event: Event) {
    match event {
        Event::Key(key_event) => handle_key_event(ui, key_event),
        _ => {}
    }
}

fn handle_key_event(ui: &mut EmulatorVisualizer, key_event: KeyEvent) {
    match key_event.kind {
        KeyEventKind::Press => handle_key_press(ui, key_event.code),
        _ => {}
    }
}

fn handle_key_press(ui: &mut EmulatorVisualizer, key_code: KeyCode) {
    match key_code {
        KeyCode::Char('q') => ui.exit(),
        KeyCode::Char('s') => ui.program.step(),
        KeyCode::Char('r') => ui.program.reset(),
        _ => {}
    }
}
