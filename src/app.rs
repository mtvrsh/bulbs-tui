use anyhow::{Context, Result};
use std::io;
use std::{fs, path::PathBuf};

use crate::api::{self, Device, Devices};

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
    pub devices: Devices,
    pub logs: Vec<String>,
    config_path: PathBuf,

    pub current_device_index: usize,
    pub current_widget: CurrentWidget,
    pub currently_adding: Option<CurrentlyAdding>,
    pub currently_setting: Option<CurrentlySetting>,

    pub log_horizontal_offset: u16,
    pub color_input: String,
    pub brightness_input: String,
    pub ip_input: String,
    pub name_input: String,
}

macro_rules! log {
    ($app:expr, $line:expr) => {{
        $app.logs.push($line);
    }};
}

impl App {
    pub fn new(config: Devices, path: PathBuf) -> Self {
        Self {
            devices: config,
            logs: Vec::default(),
            config_path: path,

            current_device_index: 0,
            current_widget: CurrentWidget::Devices,
            currently_adding: None,
            currently_setting: None,

            log_horizontal_offset: 0,
            color_input: String::new(),
            brightness_input: String::new(),
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

    pub fn toggle_settings_field(&mut self) {
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
        if let Some(first) = self.devices.bulbs.iter().find(|d| d.selected) {
            self.color_input = first.bulb.color.to_string();
            self.brightness_input = first.bulb.brightness.to_string();
            self.current_widget = CurrentWidget::DeviceSettings;
            self.currently_setting = Some(CurrentlySetting::Color);
        }
    }

    pub fn scroll_logs_left(&mut self) {
        self.log_horizontal_offset = self.log_horizontal_offset.saturating_sub(4);
    }
    pub fn scroll_logs_right(&mut self) {
        self.log_horizontal_offset = self.log_horizontal_offset.saturating_add(4);
    }

    pub fn write_config(&mut self) -> Result<()> {
        let devices = toml::to_string(&self.devices)?;
        fs::write(self.config_path.as_path(), devices).with_context(|| {
            format!(
                "failed to write config: {}",
                self.config_path.to_string_lossy()
            )
        })?;
        Ok(())
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
        if !self.devices.bulbs.is_empty() {
            self.devices.bulbs.remove(self.current_device_index);
            self.prev_device();
        }
    }

    pub fn add_device(&mut self) {
        if self.devices.bulbs.iter().any(|x| x.ip == self.ip_input) {
            log!(self, format!("Device \"{}\" already added", self.ip_input));
            return;
        }
        if !self.ip_input.is_empty() {
            match self
                .devices
                .add(self.ip_input.clone(), self.name_input.clone())
            {
                Ok(v) => log!(self, v),
                Err(e) => {
                    log!(self, e.to_string());
                    return;
                }
            }
            self.ip_input.clear();
            self.name_input.clear();
        }
        self.currently_adding = None;
        self.current_widget = CurrentWidget::Devices;
    }

    pub fn refresh_devices(&mut self) {
        if self.devices.bulbs.is_empty() {
            return;
        }
        match self.devices.get_status() {
            Ok(None) => (),
            Ok(Some(v)) => log!(self, v),
            Err(e) => log!(self, e.to_string()),
        }
    }

    pub fn discover(&mut self) {
        match api::discover_bulbs(200) {
            Ok(v) => {
                if v.is_empty() {
                    log!(self, "No devices detected".to_string());
                }
                for ip in v {
                    if !self.devices.bulbs.iter().any(|x| x.ip == ip) {
                        match self.devices.add(ip, String::new()) {
                            Ok(v) => log!(self, v),
                            Err(e) => {
                                log!(self, e.to_string());
                                return;
                            }
                        }
                    }
                }
            }
            Err(e) => log!(self, e.to_string()),
        }
    }

    pub fn toggle_selected(&mut self) {
        match self.devices.toggle() {
            Ok(()) => (),
            Err(e) => log!(self, e.to_string()),
        }
    }

    pub fn toggle_current(&mut self) {
        if !self.devices.bulbs.is_empty() {
            if let Err(e) = self.devices.toggle_one(self.current_device_index) {
                log!(self, e.to_string());
            }
        }
    }

    pub fn set_color_and_brightness(&mut self) {
        if !self.color_input.is_empty() && self.color_input.len() == 7 {
            if let Err(e) = self.devices.set_color(&self.color_input) {
                log!(self, e.to_string());
                return;
            }
        } else {
            log!(self, format!("failed to set color: wrong input lenght"));
            return;
        }

        let brightness = self.brightness_input.parse::<f32>();
        match brightness {
            Ok(v) => {
                // compare floats with error margin, ty clippy
                let error_margin = f32::EPSILON;
                if (v - self.current_device().bulb.brightness).abs() > error_margin {
                    if let Err(e) = self.devices.set_brightness(v) {
                        log!(self, e.to_string());
                    }
                }
            }
            Err(e) => {
                log!(self, format!("failed to set brightness: {e}"));
                return;
            }
        }

        self.currently_setting = None;
        self.current_widget = CurrentWidget::Devices;
    }
}

pub fn load_devices(path: PathBuf) -> Result<Devices> {
    let cfg = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                return Ok(Devices::new());
            }
            return Err(e.into());
        }
    };
    toml::from_str(cfg.as_str()).map_err(std::convert::Into::into)
}
