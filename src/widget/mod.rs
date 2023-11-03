use std::{any::Any, collections::HashMap, marker::PhantomData, time::Instant};

use chrono::prelude::*;
use serde::{Deserialize, Serialize};

pub mod weather;

/// The unique ID of a widget (with a private value to make it non-instantiatable outside the platform itself )
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct WidgetId(String);

/// State Trait defines what is required for the widget state that will be shared with the frontend
pub trait State: Serialize {}

/// Blanket implementation for all types that implement Serialize
impl<T: Serialize> State for T {}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum Initiator {
    Schedule,
    Manual,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct RunId(pub usize);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackendRun {
    pub id: RunId,
    widget: WidgetId,
    initiated: Initiator,
    started: DateTime<Utc>,
    ended: DateTime<Utc>,
    log: String,
    result: Result<Option<String>, BackendError>,
}

/// BackendContext is provided by the backend itself and has methods to for example retrieve secrets and create notifications, read configuration, store KV-like state across reruns?
/// Functions for printing (if we cannot capture the output through the log crate)
pub struct BackendContext<'a> {
    /// The Id is needed to uniquely identify the backend/widget that is requesting the thing
    id: WidgetId,
    state: &'a mut HashMap<WidgetId, Box<dyn Any + Sync + Send>>,
}

impl<'a> BackendContext<'a> {
    pub fn get_state_or<S: Sized + Sync + Send + 'static>(&'a mut self, or: S) -> &'a mut S {
        self.state
            .entry(self.id.clone())
            .or_insert(Box::new(or))
            .downcast_mut::<S>()
            .expect("Could not downcast backend state")
    }
}

impl<C: WidgetBackend, S: State> WidgetDefinition<C, S> {
    pub fn id(&self) -> WidgetId {
        WidgetId(self.id.clone())
    }
    pub fn run(&self, state: &mut HashMap<WidgetId, Box<dyn Any + Sync + Send>>) -> BackendRun {
        let id = WidgetId(self.id.clone());

        let mut ctx = BackendContext {
            id: id.clone(),
            state,
        };

        let start = Utc::now();
        let result = self.config.run(&mut ctx);

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
}
