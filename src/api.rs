use std::{net::UdpSocket, time::Duration};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Bulb {
    #[serde(default = "default_brightness")]
    pub brightness: f32, // range: 0..1

    #[serde(default = "default_color")]
    pub color: String,

    #[serde(default, alias = "on")]
    pub enabled: u8, // api uses int instead of bool
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Device {
    #[serde(flatten)]
    pub bulb: Bulb,

    pub ip: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,

    #[serde(default = "always_true")]
    pub selected: bool,
}

const fn always_true() -> bool {
    true
}

const fn default_brightness() -> f32 {
    1.0
}

fn default_color() -> String {
    String::from("#FFFFFF")
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
        Ok(format!("{}: {}", self.ip, resp))
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
        let color = color.strip_prefix('#').unwrap_or(color);
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Devices {
    #[serde(skip, default = "default_agent")]
    agent: Agent,

    #[serde(rename = "bulb")]
    pub bulbs: Vec<Device>,
}

fn default_agent() -> Agent {
    AgentBuilder::new()
        .timeout_connect(Duration::from_secs(1))
        .timeout(Duration::from_secs(1))
        .build()
}

impl Devices {
    pub fn new() -> Self {
        Self {
            agent: default_agent(),
            bulbs: Vec::default(),
        }
    }

    pub fn add(&mut self, ip: String, name: String) -> Result<String> {
        let mut bulb = Device::new(ip, name);
        let resp = bulb.get_status(&self.agent)?;
        self.bulbs.push(bulb);
        Ok(resp)
    }

    pub fn get_status(&mut self) -> Result<Option<String>> {
        let mut resp = String::new();
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                resp.push_str(&self.bulbs[i].get_status(&self.agent)?);
            }
        }

        if resp.is_empty() {
            return Ok(None);
        }
        Ok(Some(resp))
    }

    pub fn on(&mut self) -> Result<()> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].on(&self.agent)?;
            }
        }
        Ok(())
    }

    pub fn off(&mut self) -> Result<()> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].off(&self.agent)?;
            }
        }
        Ok(())
    }

    pub fn toggle(&mut self) -> Result<()> {
        let mut first_is_enabled = 0;
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                first_is_enabled = self.bulbs[i].bulb.enabled;
                break;
            }
        }
        if first_is_enabled == 1 {
            self.off()
        } else {
            self.on()
        }
    }

    pub fn toggle_one(&mut self, index: usize) -> Result<()> {
        self.bulbs[index].toggle(&self.agent)
    }

    pub fn set_color(&mut self, color: &str) -> Result<()> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].set_color(&self.agent, color)?;
            }
        }
        Ok(())
    }

    pub fn set_brightness(&mut self, brightness: f32) -> Result<()> {
        for i in 0..self.bulbs.len() {
            if self.bulbs[i].selected {
                self.bulbs[i].set_brightness(&self.agent, brightness)?;
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
            anyhow!(
                "{url}: status code: {code}: {}",
                body.strip_suffix('\n').unwrap_or(&body)
            )
        }
        ureq::Error::Transport(_) => error.into(),
    }
}

pub fn discover_bulbs(timeout: u64) -> Result<Vec<String>> {
    const BULBS_PING: &[u8; 16] = b"bulbsclientping0";
    const BULBS_PONG: &[u8; 16] = b"bulbsserverpong0";

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_write_timeout(Some(Duration::from_millis(timeout)))?;
    socket.set_read_timeout(Some(Duration::from_millis(timeout)))?;
    socket.set_broadcast(true)?;
    socket.send_to(BULBS_PING, "255.255.255.255:5001")?;

    let mut buf = [0; BULBS_PONG.len()];
    let mut devices = Vec::<String>::new();
    loop {
        match socket.recv_from(&mut buf) {
            Ok((_, addr)) => {
                if buf == *BULBS_PONG {
                    devices.push(addr.ip().to_string());
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Ok(devices);
                }
                return Err(e.into());
            }
        }
    }
}
