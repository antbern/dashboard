//! Contains types that are shared between the backend and the frontend
//! such as Widget state definitions and the enums of all widget states etc.

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

/// The unique ID of a widget
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct WidgetId(String);

/// State Trait defines what is required for the widget state that will be shared with the frontend
pub trait State: Serialize {}

/// Blanket implementation for all types that implement Serialize
impl<T: Serialize> State for T {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WidgetDefinition<C: Serialize, S: State> {
    /// The unique ID of this widget
    pub id: WidgetId,

    /// The configuration that belongs to this widget
    pub config: C,

    #[serde(skip)]
    _state: PhantomData<S>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum WidgetEnum {
    Weather(weather::Widget),
}

/// The definitions for the weather widget
pub mod weather {
    use super::*;

    /// A test widget that returns the temperature!
    pub type Widget = WidgetDefinition<Config, Output>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Config {
        pub location: [f64; 2],
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Output {
        pub temperature: f64,
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
