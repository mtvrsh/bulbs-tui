use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    error::Error,
    io::{self, stdout},
};

mod api;
mod app;
mod rgb;
mod ui;
use crate::app::{App, CurrentWidget, CurrentlyAdding};
use crate::ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    match res {
        Ok(()) => {}
        Err(e) => eprintln!("{e}"),
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match app.current_widget {
                CurrentWidget::Devices => match key.code {
                    KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h' | 'l') => {
                        app.current_widget = CurrentWidget::Logs;
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.prev_device(),
                    KeyCode::Down | KeyCode::Char('j') => app.next_device(),
                    KeyCode::Char('a') => {
                        app.current_widget = CurrentWidget::AddDevice;
                        app.currently_adding = Some(CurrentlyAdding::IP);
                    }
                    KeyCode::Char('A') => app.discover(),
                    KeyCode::Char('c') => {
                        if !app.bulbs.devices.is_empty() {
                            app.current_widget = CurrentWidget::PickColor;
                        }
                    }
                    KeyCode::Char('d') => app.remove_device(),
                    KeyCode::Char('e') => app.toggle_selected(),
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => app.select_device(),
                    _ => {}
                },
                CurrentWidget::Logs => match key.code {
                    KeyCode::Backspace => app.logs.clear(),
                    KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h' | 'l') => {
                        app.current_widget = CurrentWidget::Devices;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                    _ => {}
                },
                CurrentWidget::AddDevice => match key.code {
                    KeyCode::Enter => app.add_device(),
                    KeyCode::Backspace => {
                        if let Some(editing) = &app.currently_adding {
                            match editing {
                                CurrentlyAdding::IP => _ = app.ip_input.pop(),
                                CurrentlyAdding::Name => _ = app.name_input.pop(),
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.current_widget = CurrentWidget::Devices;
                        app.currently_adding = None;
                    }
                    KeyCode::Tab => app.toggle_adding_field(),
                    KeyCode::Char(c) => {
                        if let Some(editing) = &app.currently_adding {
                            match editing {
                                CurrentlyAdding::IP => app.ip_input.push(c),
                                CurrentlyAdding::Name => app.name_input.push(c),
                            }
                        }
                    }
                    _ => {}
                },
                CurrentWidget::PickColor => match key.code {
                    KeyCode::Enter => app.set_color(),
                    KeyCode::Backspace => _ = app.color_input.pop(),
                    KeyCode::Esc => app.current_widget = CurrentWidget::Devices,
                    KeyCode::Char(c) => app.color_input.push(c),
                    _ => {}
                },
            }
        }
    }
}
