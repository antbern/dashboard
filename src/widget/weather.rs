use serde::{Deserialize, Serialize};

use super::{BackendContext, WidgetBackend, WidgetDefinition};

/// A test widget that returns the temperature!
pub type WeatherWidget = WidgetDefinition<Config, Output>;

#[derive(Debug, Deserialize)]
pub struct Config {
    location: [f64; 2],
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Output {
    temperature: f64,
}

#[derive(Debug)]
struct BackendState {
    value: f64,
}

impl WidgetBackend for Config {
    type Output = Output;

    fn run<'a>(
        &self,
        ctx: &'a mut BackendContext<'a>,
    ) -> Result<Option<Self::Output>, super::BackendError> {
        let state: &mut BackendState =
            ctx.get_state_or::<BackendState>(BackendState { value: 0.0 });

        dbg!(self);

        let new = Output {
            temperature: state.value,
        };

        state.value += 1.0;

        Ok(Some(new))
    }
}
