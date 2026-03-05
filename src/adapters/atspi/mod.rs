mod query;
mod session;

use std::path::Path;
use std::time::Duration;

use crate::core::command::CommandBackend;
use crate::core::model::{AppDescriptor, NodeDescriptor, ScrollDirection};
use crate::error::{AtspiCliError, Result};

pub use query::AtspiQuery;
pub use session::AtspiSession;

#[derive(Default)]
pub struct AtspiBackend {
    query: AtspiQuery,
}

impl AtspiBackend {
    pub fn new() -> Self {
        Self { query: AtspiQuery }
    }
}

impl CommandBackend for AtspiBackend {
    fn list_apps(&self) -> Result<Vec<AppDescriptor>> {
        self.query.list_applications_sync()
    }

    fn read_node(&self, locator: &str) -> Result<NodeDescriptor> {
        self.query.read_node(locator)
    }

    fn snapshot(&self, locator: &str) -> Result<String> {
        let _node = self.read_node(locator)?;
        Ok(format!("{{\"snapshot\":\"{locator}\"}}"))
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
        if self.query.read_node(locator).is_ok() {
            return Ok(());
        }
        Err(AtspiCliError::Timeout {
            locator: locator.to_string(),
            timeout_ms: timeout.as_millis() as u64,
        })
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
