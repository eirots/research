mod app;
mod ck_machine {
    pub mod ck;
}
mod cesk_machine {
    pub mod cesk;
}

mod ui;

use std::{io, time::Duration};

use crate::app::App;

// core ratatui types
use ratatui::DefaultTerminal;

// crossterm event system re-exported via ratatui
use ratatui::crossterm::event::{self, Event, KeyEventKind};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::default();

    let res = run(&mut terminal, &mut app);
    ratatui::restore();
    res
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.exit {
        terminal.draw(|f| {
            let area = f.area();
            let buf = f.buffer_mut();
            ui::render_app(area, buf, app);
        })?;

        // pull in events from crossterm
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key_event(key);
                }
            }
        }
    }
    Ok(())
}
