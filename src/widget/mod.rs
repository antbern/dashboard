use std::{any::Any, collections::HashMap, marker::PhantomData, time::Instant};

use serde::{Deserialize, Serialize};

use crate::{BackendRun, WidgetId};

pub mod weather;

// pub struct State(String);
pub trait State: Serialize {}

/// Blanket implementation for all types that implement Serialize
impl<T: Serialize> State for T {}

#[derive(Debug)]
pub enum BackendError {
    // TODO
}

/// Backend that does all the computing etc
pub trait WidgetBackend<C> {
    type State: Serialize;
    fn run<'a>(
        ctx: &'a mut BackendContext<'a>,
        config: &C,
    ) -> Result<Option<Self::State>, BackendError>;
}

// pub enum RunOutput<S> {
//     NewState(S),
//     NoState,
// }

#[derive(Debug, Deserialize)]
pub struct WidgetDefinition<C, B: WidgetBackend<C>, S: State> {
    id: String,
    config: C,

    #[serde(skip)]
    _backend: PhantomData<B>,
    #[serde(skip)]
    _state: PhantomData<S>,
}

// pub struct WidgetDefinition2<C> {
//     id: String,
//     config: C,
// }

pub struct BackendContext<'a> {
    /// The Id is needed to uniquely identify the backend/widget that is requesting the thing
    id: WidgetId,
    state: &'a mut HashMap<WidgetId, Box<dyn Any>>,
}

impl<'a> BackendContext<'a> {
    pub fn get_state_or<S: Sized + 'static>(&'a mut self, or: S) -> &'a mut S {
        self.state
            .entry(self.id)
            .or_insert(Box::new(or))
            .downcast_mut::<S>()
            .expect("Could not downcast backend state")
    }
}

impl<C, B: WidgetBackend<C>, S: State> WidgetDefinition<C, B, S> {
    pub fn run(&self, state: &mut HashMap<WidgetId, Box<dyn Any>>) -> BackendRun {
        let mut ctx = BackendContext {
            id: crate::WidgetId(0), //self.id.clone()),
            state,
        };

        let start = Instant::now();
        let result = B::run(&mut ctx, &self.config);

        let end = Instant::now();

        // serialize the returned state

        let result = result.map(|r| r.and_then(|v| Some(serde_json::to_string(&v).unwrap())));
        BackendRun {
            id: 0,
            widget: crate::WidgetId(0), //self.id.clone()),
            initiated: crate::Initiator::Manual,
            started: start,
            ended: end,
            log: "".into(),
            result: result,
        }
    }
}
