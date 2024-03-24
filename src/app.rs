use std::time::Duration;
use ureq::{Agent, AgentBuilder};

use crate::api::{Bulb, Bulbs};

pub enum CurrentWidget {
    Devices,
    Logs,
    AddDevice,
    PickColor,
}

pub enum CurrentlyAdding {
    IP,
    Name,
}

pub struct App {
    pub agent: Agent,
    pub bulbs: Bulbs,
    pub logs: Vec<String>,

    pub current_device_index: usize,
    pub current_widget: CurrentWidget,
    pub currently_adding: Option<CurrentlyAdding>,

    pub color_input: String,
    pub ip_input: String,
    pub name_input: String,
}

macro_rules! log {
    ($app:expr, $line:expr) => {{
        $app.logs.insert(0, $line);
    }};
}

impl App {
    pub fn new() -> Self {
        Self {
            agent: AgentBuilder::new()
                .timeout_connect(Duration::from_secs(1))
                .timeout(Duration::from_secs(1))
                .build(),
            bulbs: Bulbs::default(),
            logs: Vec::default(),

            current_device_index: 0,
            current_widget: CurrentWidget::Devices,
            currently_adding: None,

            color_input: String::new(),
            ip_input: String::new(),
            name_input: String::new(),
        }
    }

    pub fn toggle_adding_field(&mut self) {
        if let Some(edit_mode) = &self.currently_adding {
            match edit_mode {
                CurrentlyAdding::IP => self.currently_adding = Some(CurrentlyAdding::Name),
                CurrentlyAdding::Name => self.currently_adding = Some(CurrentlyAdding::IP),
            };
        } else {
            self.currently_adding = Some(CurrentlyAdding::IP);
        }
    }

    fn current_device(&mut self) -> &mut Bulb {
        &mut self.bulbs.devices[self.current_device_index]
    }

    pub fn prev_device(&mut self) {
        self.current_device_index = self.current_device_index.saturating_sub(1);
    }

    pub fn next_device(&mut self) {
        if self.current_device_index < self.bulbs.devices.len().saturating_sub(1) {
            self.current_device_index = self.current_device_index.saturating_add(1);
        }
    }

    pub fn select_device(&mut self) {
        if !self.bulbs.devices.is_empty() {
            self.current_device().selected = !self.current_device().selected;
        }
    }

    pub fn remove_device(&mut self) {
        self.bulbs.devices.remove(self.current_device_index);
        self.prev_device();
    }

    pub fn add_device(&mut self) {
        if self.bulbs.devices.iter().any(|x| x.ip == self.ip_input) {
            log!(
                self,
                format!("device \"{}\" already present on list", self.ip_input)
            );
            return;
        }
        if !self.ip_input.is_empty() {
            let mut bulb = Bulb::new(self.ip_input.clone(), self.name_input.clone());
            if let Err(e) = bulb.update(&self.agent) {
                log!(self, e.to_string());
                return;
            }
            if let Err(e) = bulb.on(&self.agent) {
                log!(self, e.to_string());
                return;
            }
            self.bulbs.devices.push(bulb);
            self.ip_input.clear();
            self.name_input.clear();
        }
        self.currently_adding = None;
        self.current_widget = CurrentWidget::Devices;
    }

    pub fn discover(&mut self) {
        log!(self, "discover, is not implemented".to_string());
    }

    pub fn toggle_selected(&mut self) {
        if let Err(e) = self.bulbs.toggle(&self.agent) {
            log!(self, e.to_string());
        }
    }

    pub fn set_color(&mut self) {
        if let Err(e) = self.bulbs.set_color(&self.agent, self.color_input.clone()) {
            log!(self, e.to_string());
        }
        self.color_input.clear();
        self.current_widget = CurrentWidget::Devices;
    }
}
