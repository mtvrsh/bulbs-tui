use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ureq::Agent;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
    #[serde(skip_serializing, default = "always_true")]
    pub selected: bool,
}

// for serde default value = true
const fn always_true() -> bool {
    true
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

    pub fn get_status(&mut self, agent: &Agent) -> Result<String> {
        let resp = agent
            .get(format!("http://{}/led", self.ip).as_str())
            .call()
            .map_err(with_body)?
            .into_string()?;
        self.bulb = serde_json::from_str(&resp)?;
        Ok(resp)
    }

    pub fn on(&mut self, agent: &Agent) -> Result<()> {
        agent
            .put(format!("http://{}/led/on", self.ip).as_str())
            .call()
            .map_err(with_body)?;
        self.bulb.enabled = 1;
        Ok(())
    }

    pub fn off(&mut self, agent: &Agent) -> Result<()> {
        agent
            .put(format!("http://{}/led/off", self.ip).as_str())
            .call()
            .map_err(with_body)?;
        self.bulb.enabled = 0;
        Ok(())
    }

    pub fn toggle(&mut self, agent: &Agent) -> Result<()> {
        if self.bulb.enabled == 1 {
            self.off(agent)
        } else {
            self.on(agent)
        }
    }

    pub fn set_color(&mut self, agent: &Agent, color: &str) -> Result<()> {
        agent
            .put(format!("http://{}/led/color/{}", self.ip, color).as_str())
            .call()
            .map_err(with_body)?;
        self.bulb.color = "#".to_owned() + color;
        Ok(())
    }

    pub fn set_brightness(&mut self, agent: &Agent, brightness: f32) -> Result<()> {
        agent
            .put(format!("http://{}/led/brightness/{}", self.ip, brightness).as_str())
            .call()
            .map_err(with_body)?;
        self.bulb.brightness = brightness;
        Ok(())
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
    pub fn get_status(&mut self, agent: &Agent) -> Result<String> {
        let mut resp = String::new();
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                resp.push_str(&self.bulbs[i].get_status(agent)?);
            }
        }
        Ok(resp)
    }

    pub fn on(&mut self, agent: &Agent) -> Result<()> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].on(agent)?;
            }
        }
        Ok(())
    }

    pub fn off(&mut self, agent: &Agent) -> Result<()> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].off(agent)?;
            }
        }
        Ok(())
    }

    pub fn toggle(&mut self, agent: &Agent) -> Result<()> {
        let mut first_is_enabled = 0;
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                first_is_enabled = self.bulbs[i].bulb.enabled;
                break;
            }
        }
        if first_is_enabled == 1 {
            self.off(agent)
        } else {
            self.on(agent)
        }
    }

    pub fn set_color(&mut self, agent: &Agent, color: &str) -> Result<()> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].set_color(agent, color)?;
            }
        }
        Ok(())
    }

    pub fn set_brightness(&mut self, agent: &Agent, brightness: f32) -> Result<()> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].set_brightness(agent, brightness)?;
            }
        }
        Ok(())
    }
}

/// Converts `ureq::Error` to `anyhow::Error` but with added response body.
/// Needed because donwcasting later is not possible and anyhow by default
/// doesn't display body.
fn with_body(error: ureq::Error) -> anyhow::Error {
    match error {
        ureq::Error::Status(code, response) => {
            let url = response.get_url().to_string();
            let body = match response.into_string() {
                Ok(v) => v,
                Err(e) => e.to_string(),
            };
            anyhow!("{url}: status code: {code}: {body}")
        }
        ureq::Error::Transport(_) => error.into(),
    }
}
