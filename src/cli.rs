use clap::Parser;
use std::{error::Error, ffi::OsString, path::PathBuf, time::Duration};
use ureq::AgentBuilder;

use crate::api::Device;
use crate::config::Config;

pub fn parse() -> Cli {
    Cli::parse()
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum PowerState {
    On,
    Off,
    Toggle,
}

#[derive(Debug, Parser)]
#[command(about, version)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Path to config file
    #[arg(long,  default_value=xdg_cfg_path())]
    pub config: PathBuf,

    /// Address of device to control (overrides devices defined in config file)
    #[arg(short)]
    pub addrs: Vec<String>,

    /// Set brightness
    #[arg(short, value_name = "NUM")]
    pub brightness: Option<f32>,

    /// Set color
    #[arg(short)]
    pub color: Option<String>,

    /// Show device properties
    #[arg(short)]
    pub status: bool,

    /// Set LED power
    pub power: Option<PowerState>,
}

impl Cli {
    pub fn run(&self, config: &mut Config) -> Result<(Option<String>, bool), Box<dyn Error>> {
        let mut run_tui = true;
        let mut status: Option<String> = None;
        let agent = AgentBuilder::new()
            .timeout_connect(Duration::from_secs(1))
            .timeout(Duration::from_secs(1))
            .build();

        if !self.addrs.is_empty() {
            *config = Config::default();
            for a in &self.addrs {
                config.bulbs.push(Device::new(a.into(), String::new()));
            }
        }

        if let Some(brght) = self.brightness {
            run_tui = false;
            config.set_brightness(&agent, brght)?;
            // is ^ string useful for anything?
        }
        if let Some(color) = self.color.clone() {
            run_tui = false;
            config.set_color(&agent, &color)?;
            // same deal ^^
        }
        if let Some(power) = self.power.clone() {
            run_tui = false;
            match power {
                PowerState::On => config.on(&agent)?,
                PowerState::Off => config.off(&agent)?,
                PowerState::Toggle => {
                    config.status(&agent)?;
                    config.toggle(&agent)?;
                }
            }
        }
        if self.status {
            run_tui = false;
            status = Some(config.status(&agent)?);
        }

        Ok((status, run_tui))
    }
}

fn xdg_cfg_path() -> OsString {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bulbs").expect("Failed to get XDG dirs");
    xdg_dirs
        .place_config_file("tui.toml")
        .unwrap_or_else(|_| "config.toml".into())
        .into_os_string()
}
