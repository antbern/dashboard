use common::weather::{Config, Output};

use super::{BackendContext, WidgetBackend};

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

        let new = Output {
            temperature: state.value,
        };

        state.value += 1.0;

        Ok(Some(new))
    }
}
