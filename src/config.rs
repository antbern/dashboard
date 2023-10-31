use std::fs;

use crate::widget::weather::WeatherWidget;
use anyhow::anyhow;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum WidgetEnum {
    Weather(WeatherWidget),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub widgets: Vec<WidgetEnum>,
}

pub fn load_config() -> Result<Config, anyhow::Error> {
    // read file contents
    let contents = fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&contents).map_err(|e| anyhow!(e))?;

    Ok(config)
}
