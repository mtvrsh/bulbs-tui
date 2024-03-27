use std::{fs, path::PathBuf, time::Duration};
use ureq::{Agent, AgentBuilder};

use crate::api::{Device, Devices};

pub enum CurrentWidget {
    Devices,
    Logs,
    AddDevice,
    DeviceSettings,
}

pub enum CurrentlyAdding {
    IP,
    Name,
}

pub enum CurrentlySetting {
    Color,
    Brightness,
}

pub struct App {
    pub agent: Agent,
    pub devices: Devices,
    pub logs: Vec<String>,
    config_path: PathBuf,

    pub current_device_index: usize,
    pub current_widget: CurrentWidget,
    pub currently_adding: Option<CurrentlyAdding>,
    pub currently_setting: Option<CurrentlySetting>,

    pub color_input: String,
    pub brightness_input: String,
    pub ip_input: String,
    pub name_input: String,
}

macro_rules! log {
    ($app:expr, $line:expr) => {{
        $app.logs.insert(0, $line);
    }};
}

impl App {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        let path = config_path.unwrap_or_else(xdg_cfg_path);
        Self {
            agent: AgentBuilder::new()
                .timeout_connect(Duration::from_secs(1))
                .timeout(Duration::from_secs(1))
                .build(),
            devices: Devices::default(),
            logs: Vec::default(),
            config_path: path,

            current_device_index: 0,
            current_widget: CurrentWidget::Devices,
            currently_adding: None,
            currently_setting: None,

            color_input: String::new(),
            brightness_input: String::new(),
            ip_input: String::new(),
            name_input: String::new(),
        }
    }

    pub fn toggle_curr_adding_field(&mut self) {
        if let Some(edit_mode) = &self.currently_adding {
            match edit_mode {
                CurrentlyAdding::IP => self.currently_adding = Some(CurrentlyAdding::Name),
                CurrentlyAdding::Name => self.currently_adding = Some(CurrentlyAdding::IP),
            };
        } else {
            self.currently_adding = Some(CurrentlyAdding::IP);
        }
    }

    pub fn toggle_curr_setting_field(&mut self) {
        if let Some(edit_mode) = &self.currently_setting {
            match edit_mode {
                CurrentlySetting::Color => {
                    self.currently_setting = Some(CurrentlySetting::Brightness);
                }

                CurrentlySetting::Brightness => {
                    self.currently_setting = Some(CurrentlySetting::Color);
                }
            };
        } else {
            self.currently_setting = Some(CurrentlySetting::Color);
        }
    }

    pub fn open_settings(&mut self) {
        self.color_input = self.current_device().bulb.color.clone();
        self.brightness_input = self.current_device().bulb.brightness.to_string();
        if !self.devices.bulbs.is_empty() {
            self.current_widget = CurrentWidget::DeviceSettings;
            self.currently_setting = Some(CurrentlySetting::Color);
        }
    }

    pub fn load_config(&mut self) {
        if let Ok(v) = fs::read_to_string(self.config_path.as_path()) {
            let ds: Devices = match toml::from_str(v.as_str()) {
                Ok(v) => v,
                Err(e) => {
                    log!(self, format!("failed to parse config file: {e}"));
                    return;
                }
            };
            self.devices = ds;
        };
    }

    pub fn save_config(&mut self) {
        match toml::to_string(&self.devices) {
            Ok(v) => {
                if let Err(e) = fs::write(self.config_path.as_path(), v) {
                    log!(self, format!("failed to save config file: {e}"));
                }
            }
            Err(e) => log!(self, format!("failed to deserialize config file: {e}")),
        }
    }

    fn current_device(&mut self) -> &mut Device {
        &mut self.devices.bulbs[self.current_device_index]
    }

    pub fn prev_device(&mut self) {
        self.current_device_index = self.current_device_index.saturating_sub(1);
    }

    pub fn next_device(&mut self) {
        if self.current_device_index < self.devices.bulbs.len().saturating_sub(1) {
            self.current_device_index = self.current_device_index.saturating_add(1);
        }
    }

    pub fn select_device(&mut self) {
        if !self.devices.bulbs.is_empty() {
            self.current_device().selected = !self.current_device().selected;
        }
    }

    pub fn remove_device(&mut self) {
        self.devices.bulbs.remove(self.current_device_index);
        self.prev_device();
    }

    pub fn add_device(&mut self) {
        if self.devices.bulbs.iter().any(|x| x.ip == self.ip_input) {
            log!(
                self,
                format!("device \"{}\" already present on list", self.ip_input)
            );
            return;
        }
        if !self.ip_input.is_empty() {
            let mut bulb = Device::new(self.ip_input.clone(), self.name_input.clone());
            if let Err(e) = bulb.update(&self.agent) {
                log!(self, e.to_string());
                return;
            }
            if let Err(e) = bulb.on(&self.agent) {
                log!(self, e.to_string());
                return;
            }
            self.devices.bulbs.push(bulb);
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
        if let Err(e) = self.devices.toggle(&self.agent) {
            log!(self, e.to_string());
        }
    }

    pub fn toggle_current(&mut self) {
        if let Err(e) = self.devices.bulbs[self.current_device_index].toggle(&self.agent) {
            log!(self, e.to_string());
        }
    }

    pub fn set_color_brightness(&mut self) {
        if !self.color_input.is_empty() {
            if let Err(e) = self
                .devices
                .set_color(&self.agent, self.color_input.as_str())
            {
                log!(self, e.to_string());
            }
            self.color_input.clear();
        }

        let brightness = self.brightness_input.parse::<f32>();
        match brightness {
            Err(e) => {
                log!(self, e.to_string());
                return;
            }
            Ok(v) => {
                // compare floats with error margin, ty clippy
                let error_margin = f32::EPSILON;
                if (v - self.current_device().bulb.brightness).abs() > error_margin {
                    if let Err(e) = self.devices.set_brightness(&self.agent, v) {
                        log!(self, e.to_string());
                    }
                    self.brightness_input.clear();
                }
            }
        }

        self.currently_setting = None;
        self.current_widget = CurrentWidget::Devices;
    }
}

fn xdg_cfg_path() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bulbs").expect("failed to get XDG dirs");
    xdg_dirs
        .place_config_file("tui.toml")
        .expect("failed to setup XDG CONFIG dir")
}
