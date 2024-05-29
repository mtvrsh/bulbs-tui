use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{ffi::OsString, path::PathBuf};

use crate::api::{self, Device, Devices};

pub fn parse() -> Args {
    Args::parse()
}

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    /// Path to config file
    #[arg(long,  default_value=xdg_cfg_path())]
    pub config: PathBuf,

    #[command(subcommand)]
    pub cmd: Option<Subcmd>,
}

#[derive(Debug, Subcommand)]
pub enum Subcmd {
    /// Control bulbs non interactively
    Cli(Cli),
}

#[derive(clap::Args, Debug)]
pub struct Cli {
    /// Device address (can be specified mulitiple times)
    #[arg(short, value_name = "ADDR")]
    addrs: Vec<String>,

    /// Set brightness
    #[arg(short, value_name = "NUM")]
    brightness: Option<f32>,

    /// Set color
    #[arg(short)]

    /// Automatically find devices
    #[arg(short)]
    discover: bool,

    color: Option<String>,

    /// Show status
    #[arg(short)]
    status: bool,

    /// Set LED power
    power: Option<PowerState>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum PowerState {
    On,
    Off,
    Toggle,
}

impl Cli {
    pub fn run(&self, devices: &mut Devices) -> Result<Option<String>> {
        let mut status: Option<String> = None;

        if !self.addrs.is_empty() {
            *devices = Devices::new();
            for a in &self.addrs {
                devices.bulbs.push(Device::new(a.into(), String::new()));
            }
        }

        if self.discover {
            let discovered_devices = api::discover_bulbs(200)?;
            for a in discovered_devices {
                devices.bulbs.push(Device::new(a, String::new()));
            }
        }

        if devices.bulbs.is_empty() {
            return Err(CliError::NoDevicesError.into());
        }

        let mut sth_was_done = false;
        if let Some(brght) = self.brightness {
            sth_was_done = true;
            devices.set_brightness(brght)?;
        }
        if let Some(color) = self.color.clone() {
            sth_was_done = true;
            devices.set_color(&color)?;
        }
        if let Some(power) = self.power.clone() {
            sth_was_done = true;
            match power {
                PowerState::On => devices.on()?,
                PowerState::Off => devices.off()?,
                PowerState::Toggle => {
                    devices.get_status()?;
                    devices.toggle()?;
                }
            }
        }
        if self.status {
            sth_was_done = true;
            status = Some(devices.get_status()?);
        }

        if sth_was_done {
            Ok(status)
        } else {
            Err(CliError::NothingToDoError.into())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CliError {
    NoDevicesError,
    NothingToDoError,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NoDevicesError => {
                write!(f, "no devices found, provide at least one device address")
            }
            Self::NothingToDoError => write!(
                f,
                "nothing to do, provide argument or option that does something"
            ),
        }
    }
}

impl std::error::Error for CliError {}

fn xdg_cfg_path() -> OsString {
    #[allow(clippy::expect_used)]
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bulbs").expect("failed to get XDG dirs");
    xdg_dirs
        .place_config_file("tui.toml")
        .unwrap_or_else(|_| "config.toml".into())
        .into_os_string()
}
