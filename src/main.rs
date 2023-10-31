// #![allow(unused, dead_code)]
use std::{any::Any, collections::HashMap, fmt::Debug, fs};

use anyhow::anyhow;
use serde::Deserialize;
use widget::weather::WeatherWidget;
mod widget;

/// Context is provided by the backend itself and has methods to for example retrieve secrets and create notifications, read configuration, store KV-like state across reruns?
/// Functions for printing (if we cannot capture the output through the log crate)

/// The unique ID of a widget (with a private value to make it non-instantiatable outside the platform itself )
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct WidgetId(usize);

// trait State: serde::Serialize + serde::Deserialize {}
/// Yew component
/// TODO might need to be a struct
trait WidgetFrontend {
    type State: Debug;

    fn component(state: Self::State);
}

// TODO: how can we connect the backend, frontend and the state in a good way?

#[derive(Debug, Deserialize)]
enum WidgetEnum {
    Weather(WeatherWidget),
}

#[derive(Debug, Deserialize)]
struct Config {
    widgets: Vec<WidgetEnum>,
}

fn main() -> Result<(), anyhow::Error> {
    // read file contents
    let contents = fs::read_to_string("config.yaml")?;

    let config: Config = serde_yaml::from_str(&contents).map_err(|e| anyhow!(e))?;

    // can we convert it to a dyn Any

    // dbg!(&config);

    // map with any for storing configuration?
    let mut backend_state: HashMap<WidgetId, Box<dyn Any>> = HashMap::new();

    for w in &config.widgets {
        let run = match w {
            WidgetEnum::Weather(w) => w.run(&mut backend_state),
        };

        dbg!(&run);
    }

    Ok(())
}
