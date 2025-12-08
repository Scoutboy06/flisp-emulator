use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

use crate::emulator::Emulator;

pub fn handle_event(program: &mut Emulator, event: Event) {
    match event {
        Event::Key(key_event) => handle_key_event(program, key_event),
        _ => {}
    }
}

fn handle_key_event(program: &mut Emulator, key_event: KeyEvent) {
    match key_event.kind {
        KeyEventKind::Press => handle_key_press(program, key_event.code),
        _ => {}
    }
}

fn handle_key_press(program: &mut Emulator, key_code: KeyCode) {
    match key_code {
        KeyCode::Char('q') => program.exit(),
        KeyCode::Char('s') => program.step(),
        KeyCode::Char('r') => program.reset(),
        _ => {}
    }
}
