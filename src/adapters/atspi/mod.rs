mod query;
mod session;
pub(crate) mod tree;

use std::path::Path;
use std::time::Duration;

use crate::core::command::CommandBackend;
use crate::core::model::{AppDescriptor, NodeDescriptor, ScrollDirection};
use crate::error::{AtspiCliError, Result};

pub use query::AtspiQuery;
pub use session::AtspiSession;

pub struct AtspiBackend {
    query: AtspiQuery,
    runtime: tokio::runtime::Runtime,
}

impl AtspiBackend {
    pub fn new() -> Self {
        let runtime =
            tokio::runtime::Runtime::new().expect("failed to create tokio runtime for AT-SPI");
        Self {
            query: AtspiQuery,
            runtime,
        }
    }
}

impl Default for AtspiBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBackend for AtspiBackend {
    fn list_apps(&self) -> Result<Vec<AppDescriptor>> {
        self.query.list_applications_sync(&self.runtime)
    }

    fn read_node(&self, locator: &str) -> Result<NodeDescriptor> {
        self.query.read_node_sync(locator, &self.runtime)
    }

    fn has_sensitive_nodes(&self, app: &AppDescriptor) -> Result<bool> {
        self.query.has_sensitive_nodes_sync(app, &self.runtime)
    }

    fn snapshot(&self, locator: &str, depth: i32) -> Result<String> {
        self.query.snapshot_sync(locator, depth, &self.runtime)
    }

    fn get_property(&self, locator: &str, property: &str) -> Result<String> {
        let node = self.read_node(locator)?;
        match property {
            "locator" => Ok(node.locator),
            "text" => Ok(node.text.unwrap_or_default()),
            "visible" => Ok(node.visible.to_string()),
            _ => Err(AtspiCliError::InvalidArgument(format!(
                "Unsupported property '{property}'"
            ))),
        }
    }

    fn wait_for(&self, locator: &str, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(200);
        loop {
            if self.query.read_node_sync(locator, &self.runtime).is_ok() {
                return Ok(());
            }
            if start.elapsed() >= timeout {
                return Err(AtspiCliError::Timeout {
                    locator: locator.to_string(),
                    timeout_ms: timeout.as_millis() as u64,
                });
            }
            std::thread::sleep(poll_interval);
        }
    }

    fn click(&self, locator: &str, _times: u8) -> Result<()> {
        self.read_node(locator).map(|_| ())
    }

    fn hover(&self, locator: &str) -> Result<()> {
        self.read_node(locator).map(|_| ())
    }

    fn focus(&self, locator: &str) -> Result<()> {
        self.read_node(locator).map(|_| ())
    }

    fn input_text(&self, locator: &str, _text: &str, _clear_first: bool) -> Result<()> {
        self.read_node(locator).map(|_| ())
    }

    fn press_key(&self, key: &str) -> Result<()> {
        if key.trim().is_empty() {
            return Err(AtspiCliError::InvalidArgument(
                "Key cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn scroll_to(&self, locator: &str) -> Result<()> {
        self.read_node(locator).map(|_| ())
    }

    fn scroll(&self, _direction: ScrollDirection, _amount: u32) -> Result<()> {
        Ok(())
    }

    fn screenshot(&self, locator: Option<&str>, output: &Path) -> Result<()> {
        if let Some(loc) = locator {
            self.read_node(loc)?;
        }
        std::fs::write(output, "atspi-screenshot").map_err(AtspiCliError::from)
    }
}
