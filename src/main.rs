// #![allow(unused, dead_code)]

use std::{any::Any, collections::HashMap};

use widget::WidgetId;

use crate::config::WidgetEnum;

mod config;
mod widget;

fn main() -> anyhow::Result<()> {
    let config = config::load_config()?;
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
