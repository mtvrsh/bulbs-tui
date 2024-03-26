use app::CurrentlySetting;
use clap::Parser;
use crossterm::{
    event::{Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io, path::PathBuf};

mod api;
mod app;
mod rgb;
mod ui;
use crate::app::{App, CurrentWidget, CurrentlyAdding};

#[derive(Parser)]
#[command(about, version)]
struct Cli {
    /// Path to config file
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg_path = Cli::parse().config;

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new(cfg_path);
    app.load_config();
    let res = run_app(&mut terminal, &mut app);

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    match res {
        Ok(()) => {}
        Err(e) => eprintln!("{e}"),
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if let Event::Key(key) = crossterm::event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match app.current_widget {
                CurrentWidget::Devices => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
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
                    KeyCode::Char('c') => app.open_settings(),
                    KeyCode::Char('d') => app.remove_device(),
                    KeyCode::Char('e') => app.toggle_selected(),
                    KeyCode::Char('r') => app.load_config(),
                    KeyCode::Char('s') => app.save_config(),
                    KeyCode::Char(' ') => app.select_device(),
                    _ => {}
                },
                CurrentWidget::Logs => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                    KeyCode::Backspace => app.logs.clear(),
                    KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h' | 'l') => {
                        app.current_widget = CurrentWidget::Devices;
                    }
                    _ => {}
                },
                CurrentWidget::AddDevice => match key.code {
                    KeyCode::Enter => app.add_device(),
                    KeyCode::Backspace => {
                        if let Some(editing) = &app.currently_adding {
                            match editing {
                                CurrentlyAdding::IP => app.ip_input.pop(),
                                CurrentlyAdding::Name => app.name_input.pop(),
                            };
                        }
                    }
                    KeyCode::Esc => {
                        app.current_widget = CurrentWidget::Devices;
                        app.currently_adding = None;
                    }
                    KeyCode::Tab => app.toggle_curr_adding_field(),
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
                CurrentWidget::DeviceSettings => match key.code {
                    KeyCode::Enter => app.set_color_brightness(),
                    KeyCode::Backspace => {
                        if let Some(setting) = &app.currently_setting {
                            match setting {
                                CurrentlySetting::Color => app.color_input.pop(),
                                CurrentlySetting::Brightness => app.brightness_input.pop(),
                            };
                        }
                    }
                    KeyCode::Esc => {
                        app.current_widget = CurrentWidget::Devices;
                        app.currently_setting = None;
                    }
                    KeyCode::Tab => app.toggle_curr_setting_field(),
                    KeyCode::Char(c) => {
                        if let Some(setting) = &app.currently_setting {
                            match setting {
                                CurrentlySetting::Color => app.color_input.push(c),
                                CurrentlySetting::Brightness => app.brightness_input.push(c),
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}
