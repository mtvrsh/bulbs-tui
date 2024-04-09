use serde::{Deserialize, Serialize};
use std::error::Error;
use ureq::Agent;

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

    pub fn update(&mut self, agent: &Agent) -> Result<String, Box<dyn Error>> {
        let resp = agent
            .get(format!("http://{}/led", self.ip).as_str())
            .call()?;
        let s = resp.into_string()?;
        self.bulb = serde_json::from_str(&s)?;
        Ok(s)
    }

    pub fn on(&mut self, agent: &Agent) -> Result<(), Box<dyn Error>> {
        agent
            .put(format!("http://{}/led/on", self.ip).as_str())
            .call()?;
        self.bulb.enabled = 1;
        Ok(())
    }

    pub fn off(&mut self, agent: &Agent) -> Result<(), Box<dyn Error>> {
        agent
            .put(format!("http://{}/led/off", self.ip).as_str())
            .call()?
            .into_string()?;
        self.bulb.enabled = 0;
        Ok(())
    }

    pub fn toggle(&mut self, agent: &Agent) -> Result<(), Box<dyn Error>> {
        if self.bulb.enabled == 1 {
            self.off(agent)
        } else {
            self.on(agent)
        }
    }

    pub fn set_color(&mut self, agent: &Agent, color: &str) -> Result<String, Box<dyn Error>> {
        let s = agent
            .put(format!("http://{}/led/color/{}", self.ip, color).as_str())
            .call()?
            .into_string()?;
        self.bulb.color = "#".to_owned() + color;
        Ok(s)
    }

    pub fn set_brightness(
        &mut self,
        agent: &Agent,
        brightness: f32,
    ) -> Result<String, Box<dyn Error>> {
        let s = agent
            .put(format!("http://{}/led/brightness/{}", self.ip, brightness).as_str())
            .call()?
            .into_string()?;
        self.bulb.brightness = brightness;
        Ok(s)
    }
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {:16} {:16} {:3} {:4} {:7}",
            if self.selected { ">" } else { " " },
            self.ip,
            self.name,
            if self.bulb.enabled == 1 { "ON" } else { "OFF" },
            self.bulb.brightness,
            self.bulb.color,
        )
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Devices {
    #[serde(rename = "bulb")]
    pub bulbs: Vec<Device>,
}

impl Devices {
    pub fn update(&mut self, agent: &Agent) -> Result<String, Box<dyn Error>> {
        let mut s = String::new();
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                s.push_str(&self.bulbs[i].update(agent)?);
            }
        }
        Ok(s)
    }

    pub fn on(&mut self, agent: &Agent) -> Result<(), Box<dyn Error>> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].on(agent)?;
            }
        }
        Ok(())
    }

    pub fn off(&mut self, agent: &Agent) -> Result<(), Box<dyn Error>> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].off(agent)?;
            }
        }
        Ok(())
    }

    pub fn toggle(&mut self, agent: &Agent) -> Result<(), Box<dyn Error>> {
        let mut first = 0;
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                first = self.bulbs[i].bulb.enabled;
                break;
            }
        }
        if first == 1 {
            self.off(agent)
        } else {
            self.on(agent)
        }
    }

    pub fn set_color(&mut self, agent: &Agent, color: &str) -> Result<String, Box<dyn Error>> {
        let mut s = String::new();
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                s.push_str(&self.bulbs[i].set_color(agent, color)?);
            }
        }
        Ok(s)
    }

    pub fn set_brightness(
        &mut self,
        agent: &Agent,
        brightness: f32,
    ) -> Result<String, Box<dyn Error>> {
        let mut s = String::new();
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                s.push_str(&self.bulbs[i].set_brightness(agent, brightness)?);
            }
        }
        Ok(s)
    }
}

// for serde default value = true
const fn tt() -> bool {
    true
}
