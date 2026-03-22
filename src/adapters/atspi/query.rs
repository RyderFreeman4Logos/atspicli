use crate::core::model::{AppDescriptor, NodeDescriptor};
use crate::error::{AtspiCliError, Result};

use super::tree;

#[derive(Default)]
pub struct AtspiQuery;

impl AtspiQuery {
    pub fn list_applications_sync(
        &self,
        runtime: &tokio::runtime::Runtime,
    ) -> Result<Vec<AppDescriptor>> {
        #[cfg(debug_assertions)]
        if let Some(parsed) = Self::parse_fake_apps_from_env() {
            return Ok(parsed);
        }

        runtime
            .block_on(self.list_applications_async())
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    /// Query the AT-SPI registry for all running accessible applications.
    async fn list_applications_async(&self) -> std::result::Result<Vec<AppDescriptor>, zbus::Error> {
        use atspi::connection::AccessibilityConnection;
        use atspi::proxy::accessible::AccessibleProxy;

        let conn = AccessibilityConnection::open().await?;
        let root = AccessibleProxy::builder(conn.connection())
            .destination("org.a11y.atspi.Registry")?
            .path("/org/a11y/atspi/accessible/root")?
            .build()
            .await?;

        let children = root.get_children().await?;
        let mut apps = Vec::new();

        for child in &children {
            let child_proxy = AccessibleProxy::builder(conn.connection())
                .destination(child.name.as_str())?
                .path(child.path.as_ref())?
                .build()
                .await?;

            let name = child_proxy.name().await.unwrap_or_default();

            // Obtain PID via D-Bus connection credentials, falling back to 0.
            let pid = conn
                .connection()
                .call_method(
                    Some("org.freedesktop.DBus"),
                    "/org/freedesktop/DBus",
                    Some("org.freedesktop.DBus"),
                    "GetConnectionUnixProcessID",
                    &(child.name.as_str(),),
                )
                .await
                .and_then(|reply| reply.body::<u32>())
                .unwrap_or(0);

            if !name.is_empty() {
                apps.push(AppDescriptor::new(name, pid));
            }
        }

        Ok(apps)
    }

    pub fn has_sensitive_nodes_sync(
        &self,
        app: &AppDescriptor,
        runtime: &tokio::runtime::Runtime,
    ) -> Result<bool> {
        runtime
            .block_on(self.has_sensitive_nodes_async(app))
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    async fn has_sensitive_nodes_async(
        &self,
        _app: &AppDescriptor,
    ) -> std::result::Result<bool, zbus::Error> {
        use atspi::connection::AccessibilityConnection;
        use atspi::proxy::accessible::AccessibleProxy;

        let conn = AccessibilityConnection::open().await?;
        let root = AccessibleProxy::builder(conn.connection())
            .destination("org.a11y.atspi.Registry")?
            .path("/org/a11y/atspi/accessible/root")?
            .build()
            .await?;

        // Search all app roots for sensitive nodes
        let children = root.get_children().await?;
        for child in &children {
            let child_proxy = match AccessibleProxy::builder(conn.connection())
                .destination(child.name.as_str())
                .and_then(|b| b.path(child.path.as_ref()))
            {
                Ok(b) => match b.build().await {
                    Ok(p) => p,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };
            if tree::has_any_sensitive(conn.connection(), &child_proxy).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[cfg(debug_assertions)]
    fn parse_fake_apps_from_env() -> Option<Vec<AppDescriptor>> {
        let raw = std::env::var("ATSPICLI_FAKE_APPS").ok()?;
        let parsed = raw
            .split(',')
            .filter_map(|entry| {
                let (name, pid_str) = entry.split_once(':')?;
                let pid = pid_str.parse::<u32>().ok()?;
                Some(AppDescriptor::new(name.trim(), pid))
            })
            .collect::<Vec<AppDescriptor>>();
        if parsed.is_empty() {
            return None;
        }
        Some(parsed)
    }

    pub fn read_node_sync(
        &self,
        locator: &str,
        runtime: &tokio::runtime::Runtime,
    ) -> Result<NodeDescriptor> {
        if locator.trim().is_empty() {
            return Err(AtspiCliError::InvalidLocator(
                "Locator cannot be empty".to_string(),
            ));
        }

        // Locator-level sensitive check (fast path, no AT-SPI needed)
        if Self::locator_looks_sensitive(locator) {
            let mut node = NodeDescriptor::new(locator);
            node.sensitive = true;
            node.text = Some("<hidden>".to_string());
            return Ok(node);
        }

        runtime
            .block_on(self.read_node_async(locator))
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    async fn read_node_async(
        &self,
        locator: &str,
    ) -> std::result::Result<NodeDescriptor, zbus::Error> {
        use atspi::connection::AccessibilityConnection;
        use atspi::proxy::accessible::AccessibleProxy;

        let conn = AccessibilityConnection::open().await?;
        let root = AccessibleProxy::builder(conn.connection())
            .destination("org.a11y.atspi.Registry")?
            .path("/org/a11y/atspi/accessible/root")?
            .build()
            .await?;

        // Search all applications for the matching node
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

            if let Ok(Some(tree_node)) =
                tree::find_node(conn.connection(), &child_proxy, locator, &child_bus, &child_path)
                    .await
            {
                return Ok(tree_node.to_node_descriptor(locator));
            }
        }

        Err(zbus::Error::Failure(format!(
            "Node not found for locator '{locator}'"
        )))
    }

    /// Snapshot the accessibility tree starting from the matched node.
    pub fn snapshot_sync(
        &self,
        locator: &str,
        depth: i32,
        runtime: &tokio::runtime::Runtime,
    ) -> Result<String> {
        runtime
            .block_on(self.snapshot_async(locator, depth))
            .map_err(|e| AtspiCliError::Atspi(e.to_string()))
    }

    async fn snapshot_async(
        &self,
        locator: &str,
        depth: i32,
    ) -> std::result::Result<String, zbus::Error> {
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

            if locator == "root" || locator.trim().is_empty() {
                // Snapshot from app root
                let tree_node = tree::walk_tree(
                    conn.connection(),
                    &child_proxy,
                    depth,
                    0,
                    &child_bus,
                    &child_path,
                )
                .await?;
                return serde_json::to_string_pretty(&tree_node).map_err(|e| {
                    zbus::Error::Failure(format!("JSON serialization failed: {e}"))
                });
            }

            // Find the matching node and snapshot from there
            if let Ok(Some(found)) =
                tree::find_node(conn.connection(), &child_proxy, locator, &child_bus, &child_path)
                    .await
            {
                // Re-walk from the found node's position with depth limit
                // Since we already have the tree, just re-serialize with depth
                let json = serde_json::to_string_pretty(&found).map_err(|e| {
                    zbus::Error::Failure(format!("JSON serialization failed: {e}"))
                })?;
                return Ok(json);
            }
        }

        Err(zbus::Error::Failure(format!(
            "Node not found for locator '{locator}'"
        )))
    }

    fn locator_looks_sensitive(locator: &str) -> bool {
        Self::locator_looks_sensitive_str(locator)
    }

    pub(crate) fn locator_looks_sensitive_str(input: &str) -> bool {
        let lowered = input.to_ascii_lowercase();
        const MARKERS: [&str; 9] = [
            "password",
            "secret",
            "token",
            "api_key",
            "apikey",
            "credential",
            "passcode",
            "otp",
            "pin",
        ];
        MARKERS.iter().any(|marker| lowered.contains(marker))
    }

    pub async fn smoke_check_connection() -> Result<()> {
        let _ = zbus::Connection::session().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AtspiQuery;

    #[test]
    fn test_locator_looks_sensitive_detects_common_markers() {
        assert!(AtspiQuery::locator_looks_sensitive("input[name=password]"));
        assert!(AtspiQuery::locator_looks_sensitive("field[type=api_key]"));
        assert!(AtspiQuery::locator_looks_sensitive("text:token"));
        assert!(!AtspiQuery::locator_looks_sensitive("button[text=save]"));
    }

    #[test]
    fn test_locator_looks_sensitive_str_public() {
        assert!(AtspiQuery::locator_looks_sensitive_str("password_field"));
        assert!(!AtspiQuery::locator_looks_sensitive_str("save_button"));
    }

    #[tokio::test]
    async fn test_atspi_query_smoke() {
        if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err() {
            return;
        }
        AtspiQuery::smoke_check_connection()
            .await
            .expect("session bus should be available");
    }
}
