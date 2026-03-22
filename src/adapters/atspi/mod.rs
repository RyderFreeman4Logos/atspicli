mod capture;
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

    /// Find a node by locator and return its D-Bus identity (bus_name, object_path).
    fn find_node_for_action(&self, locator: &str) -> Result<(String, String)> {
        self.runtime
            .block_on(async {
                use atspi::connection::AccessibilityConnection;
                use atspi::proxy::accessible::AccessibleProxy;

                let conn = AccessibilityConnection::open().await?;
                let root = AccessibleProxy::builder(conn.connection())
                    .destination("org.a11y.atspi.Registry")?
                    .path("/org/a11y/atspi/accessible/root")?
                    .build()
                    .await?;

                let children = root.get_children().await?;
                for child in &children {
                    let child_bus = child.name.to_string();
                    let child_path = child.path.to_string();
                    let child_proxy = match AccessibleProxy::builder(conn.connection())
                        .destination(child_bus.as_str())
                        .and_then(|b| b.path(child_path.as_str()))
                    {
                        Ok(b) => match b.build().await {
                            Ok(p) => p,
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    };
                    if let Ok(Some(tree_node)) = tree::find_node(
                        conn.connection(),
                        &child_proxy,
                        locator,
                        &child_bus,
                        &child_path,
                    )
                    .await
                    {
                        return Ok((tree_node.bus_name.clone(), tree_node.object_path.clone()));
                    }
                }
                Err(zbus::Error::Failure(format!("Node not found: '{locator}'")))
            })
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
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

    fn click(&self, locator: &str, times: u8) -> Result<()> {
        let (bus_name, path) = self.find_node_for_action(locator)?;
        self.runtime
            .block_on(async {
                let conn = atspi::connection::AccessibilityConnection::open().await?;
                let action_proxy = atspi::proxy::action::ActionProxy::builder(conn.connection())
                    .destination(bus_name.as_str())?
                    .path(path.as_str())?
                    .build()
                    .await?;
                for _ in 0..times {
                    action_proxy.do_action(0).await?;
                }
                Ok::<_, zbus::Error>(())
            })
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    fn hover(&self, locator: &str) -> Result<()> {
        let (bus_name, path) = self.find_node_for_action(locator)?;
        self.runtime
            .block_on(async {
                let conn = atspi::connection::AccessibilityConnection::open().await?;
                let comp = atspi::proxy::component::ComponentProxy::builder(conn.connection())
                    .destination(bus_name.as_str())?
                    .path(path.as_str())?
                    .build()
                    .await?;
                let (x, y, w, h) = comp.get_extents(atspi::CoordType::Screen).await?;
                let center_x = x + w / 2;
                let center_y = y + h / 2;
                let dec = atspi::proxy::device_event_controller::DeviceEventControllerProxy::new(
                    conn.connection(),
                )
                .await?;
                dec.generate_mouse_event(center_x, center_y, "abs").await?;
                Ok::<_, zbus::Error>(())
            })
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    fn focus(&self, locator: &str) -> Result<()> {
        let (bus_name, path) = self.find_node_for_action(locator)?;
        self.runtime
            .block_on(async {
                let conn = atspi::connection::AccessibilityConnection::open().await?;
                let comp = atspi::proxy::component::ComponentProxy::builder(conn.connection())
                    .destination(bus_name.as_str())?
                    .path(path.as_str())?
                    .build()
                    .await?;
                comp.grab_focus().await?;
                Ok::<_, zbus::Error>(())
            })
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    fn input_text(&self, locator: &str, text: &str, clear_first: bool) -> Result<()> {
        let (bus_name, path) = self.find_node_for_action(locator)?;
        self.runtime
            .block_on(async {
                let conn = atspi::connection::AccessibilityConnection::open().await?;
                let editable =
                    atspi::proxy::editable_text::EditableTextProxy::builder(conn.connection())
                        .destination(bus_name.as_str())?
                        .path(path.as_str())?
                        .build()
                        .await?;
                if clear_first {
                    editable.set_text_contents("").await?;
                }
                editable.set_text_contents(text).await?;
                Ok::<_, zbus::Error>(())
            })
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    fn press_key(&self, key: &str) -> Result<()> {
        if key.trim().is_empty() {
            return Err(AtspiCliError::InvalidArgument(
                "Key cannot be empty".to_string(),
            ));
        }
        self.runtime
            .block_on(async {
                let conn = atspi::connection::AccessibilityConnection::open().await?;
                let dec = atspi::proxy::device_event_controller::DeviceEventControllerProxy::new(
                    conn.connection(),
                )
                .await?;
                dec.generate_keyboard_event(
                    0,
                    key,
                    atspi::proxy::device_event_controller::KeySynthType::String,
                )
                .await?;
                Ok::<_, zbus::Error>(())
            })
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    fn scroll_to(&self, locator: &str) -> Result<()> {
        // Scroll-to: focus the element to make it visible.
        self.focus(locator)
    }

    fn scroll(&self, direction: ScrollDirection, amount: u32) -> Result<()> {
        self.runtime
            .block_on(async {
                let conn = atspi::connection::AccessibilityConnection::open().await?;
                let dec = atspi::proxy::device_event_controller::DeviceEventControllerProxy::new(
                    conn.connection(),
                )
                .await?;
                let event_name = match direction {
                    ScrollDirection::Up => "b4c",
                    ScrollDirection::Down => "b5c",
                    ScrollDirection::Left => "b6c",
                    ScrollDirection::Right => "b7c",
                };
                for _ in 0..amount {
                    dec.generate_mouse_event(0, 0, event_name).await?;
                }
                Ok::<_, zbus::Error>(())
            })
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    fn screenshot(&self, locator: Option<&str>, output: &Path) -> Result<()> {
        if let Some(loc) = locator {
            let (bus_name, path) = self.find_node_for_action(loc)?;
            let (x, y, w, h) = self
                .runtime
                .block_on(async {
                    let conn = atspi::connection::AccessibilityConnection::open().await?;
                    let comp = atspi::proxy::component::ComponentProxy::builder(conn.connection())
                        .destination(bus_name.as_str())?
                        .path(path.as_str())?
                        .build()
                        .await?;
                    comp.get_extents(atspi::CoordType::Screen).await
                })
                .map_err(|e| AtspiCliError::Atspi(e.to_string()))?;
            capture::capture_region(x, y, w, h, output)
        } else {
            capture::capture_window(output)
        }
    }
}
