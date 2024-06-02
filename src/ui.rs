use ratatui::{
    prelude::*,
    widgets::{block::Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, CurrentWidget, CurrentlyAdding, CurrentlySetting};

pub fn ui(f: &mut Frame, app: &App) {
    #[allow(clippy::cast_possible_truncation)]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(app.devices.bulbs.len() as u16 + 2),
            Constraint::Length(app.logs.len() as u16 + 2),
            Constraint::Length(1),
        ])
        .split(f.size());

    let help: Line<'_>;
    let mut log_block = Block::default().borders(Borders::ALL);
    let mut devices_block = Block::default().borders(Borders::ALL);

    match &app.current_widget {
        CurrentWidget::Devices => {
            help = Line::from(vec![
                " Add device: ".white(),
                "a".blue().bold(),
                " | Autodetect: ".white(),
                "A".blue().bold(),
                " | Change color: ".white(),
                "c".blue().bold(),
                " | Remove: ".white(),
                "d".blue().bold(),
                " | ON/OFF: ".white(),
                "e".blue().bold(),
                " | Refresh: ".white(),
                "r".blue().bold(),
                " | ON/OFF (one): ".white(),
                "<enter>".blue().bold(),
                " | Select: ".white(),
                "<space>".blue().bold(),
                " | Quit: ".white(),
                "q".blue().bold(),
            ]);
            devices_block = devices_block.border_style(Style::new().light_blue());
        }
        CurrentWidget::Logs => {
            log_block = log_block.border_style(Style::new().light_blue());
            help = Line::from(vec![
                " Clear: ".white(),
                "<backspace>".blue().bold(),
                " | Quit: ".white(),
                "q".blue().bold(),
            ]);
        }
        CurrentWidget::DeviceSettings | CurrentWidget::AddDevice => {
            help = Line::from(vec![
                " Apply: ".white(),
                "<enter>".blue().bold(),
                " | Cancel: ".white(),
                "<esc>".blue().bold(),
            ]);
        }
    }

    let mut list_items = Vec::<ListItem>::new();

    for (i, dev) in app.devices.bulbs.iter().enumerate() {
        let mut style = Style::default().bold();
        if dev.bulb.enabled == 1 {
            style = style.blue();
        } else {
            style = style.dark_gray();
        }
        if app.current_device_index == i {
            style = style.on_light_blue();
        }
        let color: Color = dev.bulb.color.parse().unwrap_or(Color::LightBlue);
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(dev.to_string(), style),
            Span::styled("  ", style),
            Span::styled("   ", style.bg(color)),
        ])));
    }

    let devices = List::new(list_items).block(devices_block.title("Devices"));

    let ll: Vec<String> = app.logs.iter().map(|l| l.replace('\n', " ")).collect();
    let llen: u16 = app.logs.len().try_into().unwrap_or_default();
    let paragraph_size = chunks[2].height.saturating_sub(4);
    let scroll = if llen >= paragraph_size {
        llen - paragraph_size
    } else {
        0
    };

    let logs = Paragraph::new(ll.join("\n"))
        .block(log_block.title("Logs"))
        .scroll((scroll.saturating_sub(2), app.log_horizontal_offset));

    let header = Paragraph::new("bulbs-tui").alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);
    f.render_widget(devices, chunks[1]);
    f.render_widget(logs, chunks[2]);
    f.render_widget(help, chunks[3]);

    match &app.current_widget {
        CurrentWidget::Devices | CurrentWidget::Logs => (),
        CurrentWidget::DeviceSettings => render_device_settings(f, app),
        CurrentWidget::AddDevice => render_device_adding(f, app),
    }
}

fn render_device_adding(f: &mut Frame, app: &App) {
    if let Some(adding) = &app.currently_adding {
        let popup_block = Block::default().borders(Borders::NONE);

        let area = centered_rect(40, 40, f.size());
        f.render_widget(popup_block, area);

        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(area);

        let mut ip_block = Block::default().title("IP").borders(Borders::ALL);
        let mut name_block = Block::default().title("Name").borders(Borders::ALL);

        let active_style = Style::default().bg(Color::Blue).fg(Color::Black);
        match adding {
            CurrentlyAdding::IP => ip_block = ip_block.style(active_style),
            CurrentlyAdding::Name => name_block = name_block.style(active_style),
        };

        f.render_widget(Clear, popup_chunks[0]);
        f.render_widget(Clear, popup_chunks[1]);

        let key_text = Paragraph::new(app.ip_input.clone()).block(ip_block);
        f.render_widget(key_text, popup_chunks[0]);

        let value_text = Paragraph::new(app.name_input.clone()).block(name_block);
        f.render_widget(value_text, popup_chunks[1]);
    }
}

fn render_device_settings(f: &mut Frame, app: &App) {
    if let Some(setting) = &app.currently_setting {
        let popup_block = Block::default().borders(Borders::NONE);

        let area = centered_rect(40, 40, f.size());
        f.render_widget(popup_block, area);

        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(area);

        let color_indicator_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Fill(1), Constraint::Length(6)])
            .split(popup_chunks[0])[1];

        let mut color_block = Block::default().title("Color").borders(Borders::ALL);
        let mut brightness_block = Block::default().title("Brightness").borders(Borders::ALL);

        let active_style = Style::default().bg(Color::Blue).fg(Color::Black);
        match setting {
            CurrentlySetting::Color => color_block = color_block.style(active_style),
            CurrentlySetting::Brightness => brightness_block = brightness_block.style(active_style),
        };

        f.render_widget(Clear, popup_chunks[0]);
        f.render_widget(Clear, popup_chunks[1]);

        let color_text = Paragraph::new(app.color_input.clone()).block(color_block);
        f.render_widget(color_text, popup_chunks[0]);

        let brightness_text = Paragraph::new(app.brightness_input.clone()).block(brightness_block);
        f.render_widget(brightness_text, popup_chunks[1]);

        let color: Color = app.color_input.parse().unwrap_or(Color::Blue);
        f.render_widget(Block::new().bg(color), color_indicator_chunk);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
