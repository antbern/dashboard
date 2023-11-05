use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use crate::WidgetId;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct RunId(pub usize);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Initiator {
    Schedule,
    Manual,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum BackendError {
    // TODO
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackendRun {
    pub id: RunId,
    pub widget: WidgetId,
    pub initiated: Initiator,
    pub started: DateTime<Utc>,
    pub ended: DateTime<Utc>,
    pub log: String,
    pub result: Result<Option<String>, BackendError>,
}
