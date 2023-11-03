// #![allow(unused, dead_code)]

use std::{any::Any, collections::HashMap};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use widget::WidgetId;

use crate::config::WidgetEnum;

mod api;
mod config;
mod database;
mod widget;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_key_value_store=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::load_config()?;

    // map with any for storing configuration?
    let mut backend_state: HashMap<WidgetId, Box<dyn Any + Sync + Send>> = HashMap::new();

    for w in &config.widgets {
        let run = match w {
            WidgetEnum::Weather(w) => w.run(&mut backend_state),
        };

        dbg!(&run);
    }

    api::launch_api(config).await?;

    Ok(())
}
