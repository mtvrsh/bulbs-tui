use serde::{Deserialize, Serialize};
use ureq::{Agent, Error};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Bulb {
    pub brightness: f32, // range: 0..1
    pub color: String,
    #[serde(alias = "on")]
    pub enabled: u8, // api uses int instead of bool
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Device {
    #[serde(flatten)]
    pub bulb: Bulb,
    pub ip: String,
    #[serde(default)]
    pub name: String,
    #[serde(skip_serializing, default = "tt")]
    pub selected: bool,
}

impl Device {
    pub fn new(ip: String, name: String) -> Self {
        Self {
            ip,
            name,
            selected: true,
            ..Default::default()
        }
    }

    pub fn update(&mut self, agent: &Agent) -> Result<(), Error> {
        let b: Bulb = agent
            .get(format!("http://{}/led", self.ip).as_str())
            .call()?
            .into_json()?;
        self.bulb = b;
        Ok(())
    }

    pub fn on(&mut self, agent: &Agent) -> Result<(), Error> {
        agent
            .get(format!("http://{}/led/on", self.ip).as_str())
            .call()?;
        self.bulb.enabled = 1;
        Ok(())
    }

    pub fn off(&mut self, agent: &Agent) -> Result<(), Error> {
        agent
            .get(format!("http://{}/led/off", self.ip).as_str())
            .call()?;
        self.bulb.enabled = 0;
        Ok(())
    }

    pub fn toggle(&mut self, agent: &Agent) -> Result<(), Error> {
        self.update(agent)?;
        if self.bulb.enabled == 1 {
            self.off(agent)
        } else {
            self.on(agent)
        }
    }

    pub fn set_color(&mut self, agent: &Agent, color: &String) -> Result<(), Error> {
        agent
            .put(format!("http://{}/led/color/{}", self.ip, color).as_str())
            .call()?;
        self.bulb.color = color.clone();
        Ok(())
    }

    pub fn set_brightness(&mut self, agent: &Agent, brightness: f32) -> Result<(), Error> {
        agent
            .put(format!("http://{}/led/brightness/{}", self.ip, brightness).as_str())
            .call()?;
        self.bulb.brightness = brightness;
        Ok(())
    }
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {}  {}  {}  {}",
            if self.selected { ">" } else { " " },
            self.ip,
            if self.bulb.enabled == 1 { "ON" } else { "OFF" },
            self.bulb.color,
            self.name,
        )
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Devices {
    #[serde(rename = "bulb")]
    pub bulbs: Vec<Device>,
}

impl Devices {
    pub fn status(&mut self, agent: &Agent) -> Result<(), Error> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].update(agent)?;
            }
        }
        Ok(())
    }

    pub fn on(&mut self, agent: &Agent) -> Result<(), Error> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].on(agent)?;
            }
        }
        Ok(())
    }
    pub fn off(&mut self, agent: &Agent) -> Result<(), Error> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].off(agent)?;
            }
        }
        Ok(())
    }

    pub fn toggle(&mut self, agent: &Agent) -> Result<(), Error> {
        let mut first = 0;
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                first = self.bulbs[i].bulb.enabled;
                break;
            }
        }
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                if first == 1 {
                    self.bulbs[i].off(agent)?;
                } else {
                    self.bulbs[i].on(agent)?;
                }
            }
        }
        Ok(())
    }

    pub fn set_color(&mut self, agent: &Agent, color: String) -> Result<(), Error> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].set_color(agent, &color)?;
            }
        }
        Ok(())
    }

    pub fn set_brightness(&mut self, agent: &Agent, brightness: f32) -> Result<(), Error> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].set_brightness(agent, brightness)?;
            }
        }
        Ok(())
    }
}

const fn tt() -> bool {
    true
}
