use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{ffi::OsString, path::PathBuf, time::Duration};
use ureq::AgentBuilder;

use crate::{api::Device, config::Config};

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
    /// Device address (overrides devices defined in config file)
    #[arg(short, value_name = "ADDR")]
    addrs: Vec<String>,

    /// Set brightness
    #[arg(short, value_name = "NUM")]
    brightness: Option<f32>,

    /// Set color
    #[arg(short)]
    color: Option<String>,

    /// Show device properties
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
    pub fn run(&self, config: &mut Config) -> Result<Option<String>> {
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

        if config.bulbs.is_empty() {
            return Err(CliError::NoDevicesError.into());
        }

        let mut sth_was_done = false;
        if let Some(brght) = self.brightness {
            sth_was_done = true;
            config.set_brightness(&agent, brght)?;
        }
        if let Some(color) = self.color.clone() {
            sth_was_done = true;
            config.set_color(&agent, &color)?;
        }
        if let Some(power) = self.power.clone() {
            sth_was_done = true;
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
            sth_was_done = true;
            status = Some(config.status(&agent)?);
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
