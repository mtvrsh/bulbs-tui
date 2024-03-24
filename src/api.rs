use serde::{Deserialize, Serialize};
use std::fmt;
use ureq::{Agent, Error};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Bulb {
    pub brightness: f32,
    pub color: String,
    #[serde(alias = "on")]
    pub enabled: u8,

    #[serde(skip)]
    pub ip: String,
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    pub selected: bool,
}

impl Bulb {
    pub fn new(ip: String, name: String) -> Self {
        Self {
            ip,
            name,
            selected: true,
            ..Default::default()
        }
    }

    pub fn update(&mut self, agent: &Agent) -> Result<(), Error> {
        let b: Self = agent
            .get(format!("http://{}/led", self.ip).as_str())
            .call()?
            .into_json()?;
        self.brightness = b.brightness;
        self.color = b.color;
        self.enabled = b.enabled;
        Ok(())
    }

    pub fn on(&mut self, agent: &Agent) -> Result<(), Error> {
        agent
            .get(format!("http://{}/led/on", self.ip).as_str())
            .call()?;
        self.enabled = 1;
        Ok(())
    }

    pub fn off(&mut self, agent: &Agent) -> Result<(), Error> {
        agent
            .get(format!("http://{}/led/off", self.ip).as_str())
            .call()?;
        self.enabled = 0;
        Ok(())
    }

    pub fn toggle(&mut self, agent: &Agent) -> Result<(), Error> {
        self.update(agent)?;
        if self.enabled == 1 {
            self.off(agent)
        } else {
            self.on(agent)
        }
    }

    pub fn set_color(&mut self, agent: &Agent, color: &String) -> Result<(), Error> {
        agent
            .put(format!("http://{}/led/color/{}", self.ip, color).as_str())
            .call()?;
        self.color = color.clone();
        Ok(())
    }

    pub fn set_brightness(&mut self, agent: &Agent, brightness: f32) -> Result<(), Error> {
        agent
            .put(format!("http://{}/led/brightness/{}", self.ip, brightness).as_str())
            .call()?;
        self.brightness = brightness;
        Ok(())
    }
}

impl fmt::Display for Bulb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}  {}  {}  {}",
            if self.selected { ">" } else { " " },
            self.ip,
            if self.enabled == 1 { "ON" } else { "OFF" },
            self.color,
            self.name,
        )
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Bulbs {
    pub devices: Vec<Bulb>,
}

impl Bulbs {
    pub fn status(&mut self, agent: &Agent) -> Result<(), Error> {
        for i in 0..self.devices.len() {
            if self.devices[i].selected {
                self.devices[i].update(agent)?;
            }
        }
        Ok(())
    }

    pub fn on(&mut self, agent: &Agent) -> Result<(), Error> {
        for i in 0..self.devices.len() {
            if self.devices[i].selected {
                self.devices[i].on(agent)?;
            }
        }
        Ok(())
    }
    pub fn off(&mut self, agent: &Agent) -> Result<(), Error> {
        for i in 0..self.devices.len() {
            if self.devices[i].selected {
                self.devices[i].off(agent)?;
            }
        }
        Ok(())
    }

    pub fn toggle(&mut self, agent: &Agent) -> Result<(), Error> {
        let mut first = 0;
        for i in 0..self.devices.len() {
            if self.devices[i].selected {
                first = self.devices[i].enabled;
                break;
            }
        }
        for i in 0..self.devices.len() {
            if self.devices[i].selected {
                if first == 1 {
                    self.devices[i].off(agent)?;
                } else {
                    self.devices[i].on(agent)?;
                }
            }
        }
        Ok(())
    }

    pub fn set_color(&mut self, agent: &Agent, color: String) -> Result<(), Error> {
        for i in 0..self.devices.len() {
            if self.devices[i].selected {
                self.devices[i].set_color(agent, &color)?;
            }
        }
        Ok(())
    }

    pub fn set_brightness(&mut self, agent: &Agent, brightness: f32) -> Result<(), Error> {
        for i in 0..self.devices.len() {
            if self.devices[i].selected {
                self.devices[i].set_brightness(agent, brightness)?;
            }
        }
        Ok(())
    }
}
