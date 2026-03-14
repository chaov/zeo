pub mod openclaw;
pub mod nodejs;

use crate::engine::{JSEngine, JSValue, Result, EngineError};
use std::sync::Arc;

pub struct IntegrationLayer {
    engine: Arc<dyn JSEngine>,
}

impl IntegrationLayer {
    pub fn new(engine: Arc<dyn JSEngine>) -> Self {
        Self { engine }
    }
    
    pub fn setup_openclaw_compat(&self) -> Result<()> {
        openclaw::setup(self.engine.clone())
    }
    
    pub fn setup_nodejs_compat(&self) -> Result<()> {
        nodejs::setup(self.engine.clone())
    }
}