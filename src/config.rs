use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf};
use ureq::Agent;

use crate::api::Device;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "bulb")]
    pub bulbs: Vec<Device>,
}

pub fn load(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    let cfg = fs::read_to_string(path)?;
    let cfg = toml::from_str(cfg.as_str()).map_err(std::convert::Into::into);
    cfg
}

impl Config {
    pub fn status(&mut self, agent: &Agent) -> Result<String, Box<dyn Error>> {
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
