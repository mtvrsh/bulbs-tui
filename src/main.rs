mod api;
mod app;
mod cli;
mod config;
mod ui;

use app::CurrentlySetting;
use cli::Subcmd;
use crossterm::{
    event::{Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io};

use crate::app::{App, CurrentWidget, CurrentlyAdding};

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::parse();

    let mut cfg = config::load(args.config.clone()).unwrap_or_default();

    match &args.cmd {
        Some(cmd) => match &cmd {
            Subcmd::Cli(c) => match &c.run(&mut cfg) {
                Ok(v) => {
                    if let Some(s) = v {
                        print!("{s}");
                    }
                }
                Err(e) => eprintln!("error: {e}"),
            },
        },
        None => {
            crossterm::terminal::enable_raw_mode()?;
            crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
            let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

            let mut app = App::new(cfg, args.config);
            app.refresh_devices();

            let res = run_app(&mut terminal, &mut app);

            crossterm::terminal::disable_raw_mode()?;
            crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
            terminal.show_cursor()?;

            match res {
                Ok(()) => {}
                Err(e) => eprintln!("error: {e}"),
            }
        }
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if let Event::Key(key) = crossterm::event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match app.current_widget {
                CurrentWidget::Devices => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => return app.save_and_quit(),
                    KeyCode::Enter => app.toggle_current(),
                    KeyCode::Tab => {
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
                    KeyCode::Char('r') => app.refresh_devices(),
                    KeyCode::Char(' ') => app.select_device(),
                    _ => {}
                },
                CurrentWidget::Logs => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => return app.save_and_quit(),
                    KeyCode::Backspace => app.logs.clear(),
                    KeyCode::Tab => {
                        app.current_widget = CurrentWidget::Devices;
                    }
                    KeyCode::Left | KeyCode::Char('h') => app.scroll_logs_left(),
                    KeyCode::Right | KeyCode::Char('l') => app.scroll_logs_right(),
                    _ => {}
                },
                CurrentWidget::AddDevice => match key.code {
                    KeyCode::Esc => {
                        app.current_widget = CurrentWidget::Devices;
                        app.currently_adding = None;
                    }
                    KeyCode::Enter => app.add_device(),
                    KeyCode::Backspace => {
                        if let Some(editing) = &app.currently_adding {
                            match editing {
                                CurrentlyAdding::IP => app.ip_input.pop(),
                                CurrentlyAdding::Name => app.name_input.pop(),
                            };
                        }
                    }
                    KeyCode::Tab | KeyCode::Up | KeyCode::Down => app.toggle_adding_field(),
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
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.current_widget = CurrentWidget::Devices;
                        app.currently_setting = None;
                    }
                    KeyCode::Enter => app.set_color_and_brightness(),
                    KeyCode::Backspace => {
                        if let Some(setting) = &app.currently_setting {
                            match setting {
                                CurrentlySetting::Color => {
                                    if app.color_input.len() > 1 {
                                        app.color_input.pop();
                                    }
                                }
                                CurrentlySetting::Brightness => _ = app.brightness_input.pop(),
                            };
                        }
                    }
                    KeyCode::Tab | KeyCode::Up | KeyCode::Down => app.toggle_settings_field(),
                    KeyCode::Char(c) => {
                        if let Some(setting) = &app.currently_setting {
                            match setting {
                                CurrentlySetting::Color => {
                                    if app.color_input.len() < 7 {
                                        app.color_input.push(c);
                                    }
                                }
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
