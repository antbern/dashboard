/// Base module for describing a database
use std::collections::HashMap;

use common::WidgetId;

use crate::widget::{BackendRun, RunId};
// TODO: make a database specific version of the BackendRun that has the run ID in it (its an implementation specification and not needed for other logic)

pub enum DatabaseError {
    InvalidRunId,
    InvalidWidgetId,
    NoneAvailable,
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

pub trait Database {
    /// Get the run from the RunId
    fn get_run(&self, widget_id: WidgetId, run_id: RunId) -> DatabaseResult<BackendRun>;

    /// Insert a new Run into the database
    fn insert_run(&mut self, widget_id: WidgetId, run: BackendRun) -> DatabaseResult<RunId>;

    /// Returns all runs for the provided widget
    fn get_runs(&self, widget_id: WidgetId) -> DatabaseResult<Vec<BackendRun>>;

    /// Returns the most recent (completed) run for the specified widget
    fn get_last_run(&self, widget_id: WidgetId) -> DatabaseResult<BackendRun>;
}

pub struct InMemoryDatabase {
    runs: HashMap<WidgetId, Vec<BackendRun>>,
    run_id_counter: usize,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        InMemoryDatabase {
            runs: HashMap::new(),
            run_id_counter: 0,
        }
    }
}

impl Database for InMemoryDatabase {
    fn get_run(&self, widget_id: WidgetId, run_id: RunId) -> DatabaseResult<BackendRun> {
        let hits: Vec<&BackendRun> = self
            .runs
            .get(&widget_id)
            .ok_or(DatabaseError::InvalidWidgetId)
            .map(|runs| runs.iter().filter(|&run| run.id == run_id).collect())?;

        match hits.len() {
            0 => Err(DatabaseError::NoneAvailable),
            1 => Ok((*hits.first().unwrap()).clone()),
            _ => panic!("Should not have multiple runs with same ID!"),
        }
    }

    fn insert_run(&mut self, widget_id: WidgetId, mut run: BackendRun) -> DatabaseResult<RunId> {
        let runs = self.runs.entry(widget_id).or_default();

        let id = RunId(self.run_id_counter);
        run.id = id;

        runs.push(run);

        self.run_id_counter += 1;
        Ok(id)
    }

    fn get_runs(&self, widget_id: WidgetId) -> DatabaseResult<Vec<BackendRun>> {
        self.runs
            .get(&widget_id)
            .ok_or(DatabaseError::InvalidWidgetId)
            .cloned()
    }

    fn get_last_run(&self, widget_id: WidgetId) -> DatabaseResult<BackendRun> {
        self.runs
            .get(&widget_id)
            .ok_or(DatabaseError::InvalidWidgetId)
            .and_then(|runs| runs.last().ok_or(DatabaseError::NoneAvailable).cloned())
    }
}
