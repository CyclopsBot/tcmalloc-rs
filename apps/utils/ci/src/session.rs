#![deny(clippy::pedantic)]

use app::Cli;
use async_trait::async_trait;
use starbase::{AppResult, AppSession};
use tracing::debug;

#[derive(Clone)]
pub struct CyclopsSession {
    pub cli: Cli,
}

impl CyclopsSession {
    #[must_use]
    pub fn new(cli: Cli) -> Self {
        Self { cli }
    }
}

#[async_trait]
impl AppSession for CyclopsSession {
    async fn startup(&mut self) -> AppResult {
        debug!("Cyclops CLI starting up...");
        Ok(None)
    }

    async fn analyze(&mut self) -> AppResult {
        Ok(None)
    }

    async fn execute(&mut self) -> AppResult {
        Ok(None)
    }
    async fn shutdown(&mut self) -> AppResult {
        debug!("Shutting Down...");
        Ok(None)
    }
}
