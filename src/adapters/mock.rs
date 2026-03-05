use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::core::command::CommandBackend;
use crate::core::model::{AppDescriptor, NodeDescriptor, ScrollDirection};
use crate::error::{AtspiCliError, Result};

#[derive(Clone, Default)]
pub struct InMemoryBackend {
    apps: Arc<Mutex<Vec<AppDescriptor>>>,
    nodes: Arc<Mutex<HashMap<String, NodeDescriptor>>>,
    properties: Arc<Mutex<HashMap<(String, String), String>>>,
    events: Arc<Mutex<Vec<String>>>,
    focus_failures: Arc<Mutex<Vec<String>>>,
}

impl InMemoryBackend {
    pub fn demo() -> Self {
        let backend = Self::default();
        backend.add_app(AppDescriptor::new("demo-app", 4242));
        backend.add_node(NodeDescriptor::new("root"));
        let mut button = NodeDescriptor::new("button[text=Save]");
        button.text = Some("Save".to_string());
        backend.add_node(button);
        backend
    }

    pub fn add_app(&self, app: AppDescriptor) {
        self.apps.lock().expect("apps mutex").push(app);
    }

    pub fn add_node(&self, node: NodeDescriptor) {
        self.nodes
            .lock()
            .expect("nodes mutex")
            .insert(node.locator.clone(), node);
    }

    pub fn set_property(&self, locator: &str, key: &str, value: impl Into<String>) {
        self.properties
            .lock()
            .expect("properties mutex")
            .insert((locator.to_string(), key.to_string()), value.into());
    }

    pub fn set_focus_failure(&self, locator: &str) {
        self.focus_failures
            .lock()
            .expect("focus failures mutex")
            .push(locator.to_string());
    }

    pub fn take_events(&self) -> Vec<String> {
        std::mem::take(&mut *self.events.lock().expect("events mutex"))
    }
}

impl CommandBackend for InMemoryBackend {
    fn list_apps(&self) -> Result<Vec<AppDescriptor>> {
        Ok(self.apps.lock().expect("apps mutex").clone())
    }

    fn read_node(&self, locator: &str) -> Result<NodeDescriptor> {
        self.nodes
            .lock()
            .expect("nodes mutex")
            .get(locator)
            .cloned()
            .ok_or_else(|| AtspiCliError::NodeNotFound(locator.to_string()))
    }

    fn has_sensitive_nodes(&self, _app: &AppDescriptor) -> Result<bool> {
        Ok(self
            .nodes
            .lock()
            .expect("nodes mutex")
            .values()
            .any(|node| node.sensitive))
    }

    fn snapshot(&self, locator: &str) -> Result<String> {
        let node = self.read_node(locator)?;
        self.events
            .lock()
            .expect("events mutex")
            .push(format!("snapshot:{}", node.locator));
        Ok(format!(
            "{{\"locator\":\"{}\",\"visible\":{},\"text\":{}}}",
            node.locator,
            node.visible,
            node.text
                .map(|value| format!("\"{value}\""))
                .unwrap_or_else(|| "null".to_string())
        ))
    }

    fn get_property(&self, locator: &str, property: &str) -> Result<String> {
        self.read_node(locator)?;
        if let Some(value) = self
            .properties
            .lock()
            .expect("properties mutex")
            .get(&(locator.to_string(), property.to_string()))
            .cloned()
        {
            return Ok(value);
        }

        match property {
            "locator" => Ok(locator.to_string()),
            "text" => Ok(self.read_node(locator)?.text.unwrap_or_default()),
            "visible" => Ok(self.read_node(locator)?.visible.to_string()),
            _ => Err(AtspiCliError::InvalidArgument(format!(
                "Unsupported property '{property}'"
            ))),
        }
    }

    fn wait_for(&self, locator: &str, timeout: Duration) -> Result<()> {
        if self
            .nodes
            .lock()
            .expect("nodes mutex")
            .contains_key(locator)
        {
            return Ok(());
        }

        Err(AtspiCliError::Timeout {
            locator: locator.to_string(),
            timeout_ms: timeout.as_millis() as u64,
        })
    }

    fn click(&self, locator: &str, times: u8) -> Result<()> {
        self.read_node(locator)?;
        self.events
            .lock()
            .expect("events mutex")
            .push(format!("click:{locator}:{times}"));
        Ok(())
    }

    fn hover(&self, locator: &str) -> Result<()> {
        self.read_node(locator)?;
        self.events
            .lock()
            .expect("events mutex")
            .push(format!("hover:{locator}"));
        Ok(())
    }

    fn focus(&self, locator: &str) -> Result<()> {
        self.read_node(locator)?;
        let should_fail = self
            .focus_failures
            .lock()
            .expect("focus failures mutex")
            .iter()
            .any(|value| value == locator);
        if should_fail {
            return Err(AtspiCliError::Internal(format!(
                "Focus failed for locator '{locator}'"
            )));
        }
        self.events
            .lock()
            .expect("events mutex")
            .push(format!("focus:{locator}"));
        Ok(())
    }

    fn input_text(&self, locator: &str, text: &str, clear_first: bool) -> Result<()> {
        let mut nodes = self.nodes.lock().expect("nodes mutex");
        let node = nodes
            .get_mut(locator)
            .ok_or_else(|| AtspiCliError::NodeNotFound(locator.to_string()))?;

        if clear_first {
            node.text = Some(String::new());
            self.events
                .lock()
                .expect("events mutex")
                .push(format!("clear:{locator}"));
        }
        node.text = Some(text.to_string());
        self.events
            .lock()
            .expect("events mutex")
            .push(format!("input:{locator}:{text}"));
        Ok(())
    }

    fn press_key(&self, key: &str) -> Result<()> {
        if key.trim().is_empty() {
            return Err(AtspiCliError::InvalidArgument(
                "Key cannot be empty".to_string(),
            ));
        }
        self.events
            .lock()
            .expect("events mutex")
            .push(format!("press:{key}"));
        Ok(())
    }

    fn scroll_to(&self, locator: &str) -> Result<()> {
        self.read_node(locator)?;
        self.events
            .lock()
            .expect("events mutex")
            .push(format!("scroll-to:{locator}"));
        Ok(())
    }

    fn scroll(&self, direction: ScrollDirection, amount: u32) -> Result<()> {
        self.events
            .lock()
            .expect("events mutex")
            .push(format!("scroll:{direction:?}:{amount}"));
        Ok(())
    }

    fn screenshot(&self, locator: Option<&str>, output: &Path) -> Result<()> {
        if let Some(loc) = locator {
            self.read_node(loc)?;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let temp_path = std::env::temp_dir().join(format!("atspicli-{nanos}.tmp"));

        struct TempPathGuard {
            path: std::path::PathBuf,
        }
        impl Drop for TempPathGuard {
            fn drop(&mut self) {
                let _ = fs::remove_file(&self.path);
            }
        }

        let guard = TempPathGuard {
            path: temp_path.clone(),
        };
        let screenshot_text = match locator {
            Some(value) => format!("screenshot:{value}"),
            None => "screenshot:window".to_string(),
        };
        fs::write(&temp_path, screenshot_text.as_bytes())?;
        fs::copy(&temp_path, output)?;
        drop(guard);

        self.events
            .lock()
            .expect("events mutex")
            .push(format!("screenshot:{}", output.display()));
        Ok(())
    }
}
