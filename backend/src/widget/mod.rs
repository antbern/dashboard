use std::{any::Any, collections::HashMap};

use chrono::prelude::*;
use common::{
    backend::{BackendError, BackendRun, Initiator, RunId},
    State, WidgetDefinition, WidgetId,
};
use serde::Serialize;

pub mod weather;

pub struct BackendStateStorage(HashMap<WidgetId, Box<dyn Any + Send + Sync>>);

impl BackendStateStorage {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

/// Backend that does all the computing etc
pub trait WidgetBackend {
    type Output: State;
    fn run<'a>(
        &self,
        ctx: &'a mut BackendContext<'a>,
    ) -> Result<Option<Self::Output>, BackendError>;
}

/// BackendContext is provided by the backend itself and has methods to for example retrieve secrets and create notifications, read configuration, store KV-like state across reruns?
/// Functions for printing (if we cannot capture the output through the log crate)
pub struct BackendContext<'a> {
    /// The Id is needed to uniquely identify the backend/widget that is requesting the thing
    id: WidgetId,
    state: &'a mut BackendStateStorage,
}

impl<'a> BackendContext<'a> {
    pub fn get_state_or<S: Sized + Sync + Send + 'static>(&'a mut self, or: S) -> &'a mut S {
        self.state
            .0
            .entry(self.id.clone())
            .or_insert(Box::new(or))
            .downcast_mut::<S>()
            .expect("Could not downcast backend state")
    }
}

pub fn run<C: WidgetBackend + Serialize + PartialEq, S: State>(
    definition: &WidgetDefinition<C, S>,
    state: &mut BackendStateStorage,
) -> BackendRun {
    let id = definition.id.clone();

    let mut ctx = BackendContext {
        id: id.clone(),
        state,
    };

    let start = Utc::now();
    let result = definition.config.run(&mut ctx);
    let end = Utc::now();

    // serialize the returned state
    let result = result.map(|r| r.map(|v| serde_json::to_string(&v).unwrap()));

    BackendRun {
        id: RunId(0),
        widget: id,
        initiated: Initiator::Manual,
        started: start,
        ended: end,
        log: "".into(),
        result,
    }
}
