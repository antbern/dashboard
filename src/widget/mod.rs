use std::{any::Any, collections::HashMap, marker::PhantomData, time::Instant};

use serde::{Deserialize, Serialize};

use crate::WidgetId;

pub mod weather;

/// State Trait defines what is required for the widget state that will be shared with the frontend
pub trait State: Serialize {}

/// Blanket implementation for all types that implement Serialize
impl<T: Serialize> State for T {}

#[derive(Debug)]
pub enum BackendError {
    // TODO
}

/// Backend that does all the computing etc
pub trait WidgetBackend {
    type Output: State;
    fn run<'a>(
        &self,
        ctx: &'a mut BackendContext<'a>,
    ) -> Result<Option<Self::Output>, BackendError>;
}

#[derive(Debug, Deserialize)]
pub struct WidgetDefinition<C: WidgetBackend, S: State> {
    /// The unique ID of this widget
    id: String,

    /// The configuration that belongs to this widget
    config: C,

    #[serde(skip)]
    _state: PhantomData<S>,
}

#[derive(Debug)]
enum Initiator {
    Schedule,
    Manual,
}

#[derive(Debug)]
pub struct BackendRun {
    id: usize,
    widget: WidgetId,
    initiated: Initiator,
    started: Instant,
    ended: Instant,
    log: String,
    result: Result<Option<String>, BackendError>,
}

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

impl<C: WidgetBackend, S: State> WidgetDefinition<C, S> {
    pub fn run(&self, state: &mut HashMap<WidgetId, Box<dyn Any>>) -> BackendRun {
        let mut ctx = BackendContext {
            id: crate::WidgetId(0), //self.id.clone()),
            state,
        };

        let start = Instant::now();
        let result = self.config.run(&mut ctx);

        let end = Instant::now();

        // serialize the returned state
        let result = result.map(|r| r.and_then(|v| Some(serde_json::to_string(&v).unwrap())));

        BackendRun {
            id: 0,
            widget: crate::WidgetId(0), //self.id.clone()),
            initiated: Initiator::Manual,
            started: start,
            ended: end,
            log: "".into(),
            result,
        }
    }
}
