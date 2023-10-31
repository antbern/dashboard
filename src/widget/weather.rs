use std::any::Any;

use serde::{Deserialize, Serialize};

use super::{BackendContext, State, WidgetBackend};

#[derive(Debug)]
pub struct WeatherWidget {}

#[derive(Debug, Deserialize)]
pub struct WeatherWidgetConfig {
    location: [f64; 2],
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WeatherWidgetState {
    temperature: f64,
}

#[derive(Debug)]
struct BackendState {
    value: f64,
}

impl WidgetBackend<WeatherWidgetConfig> for WeatherWidget {
    type State = WeatherWidgetState;

    fn run<'a>(
        ctx: &'a mut BackendContext<'a>,
        config: &WeatherWidgetConfig,
    ) -> Result<Option<Self::State>, super::BackendError> {
        let state: &mut BackendState =
            ctx.get_state_or::<BackendState>(BackendState { value: 0.0 });

        dbg!(config);

        let new = WeatherWidgetState {
            temperature: state.value,
        };

        state.value += 1.0;

        Ok(Some(new))
    }
}

// #[Backend(Weather)]
// fn weather_backend<'a>(
//     ctx: &'a mut BackendContext<'a>,
//     config: &WeatherWidgetConfig,
// ) -> Result<Option<Box<dyn Any>>, super::BackendError> {
//     let state: &mut BackendState = ctx.get_state_or::<BackendState>(BackendState { value: 0.0 });

//     dbg!(config);

//     let new = WeatherWidgetState {
//         temperature: state.value,
//     };

//     state.value += 1.0;

//     Ok(Some(Box::new(new)))
// }
