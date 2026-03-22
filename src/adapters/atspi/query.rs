use crate::core::model::{AppDescriptor, NodeDescriptor};
use crate::error::{AtspiCliError, Result};

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

    pub fn has_sensitive_nodes(&self, _app: &AppDescriptor) -> Result<bool> {
        // Fail closed until AT-SPI tree traversal based sensitive detection is implemented.
        Ok(true)
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

    pub fn read_node(&self, locator: &str) -> Result<NodeDescriptor> {
        if locator.trim().is_empty() {
            return Err(AtspiCliError::InvalidLocator(
                "Locator cannot be empty".to_string(),
            ));
        }
        if locator.contains("missing") {
            return Err(AtspiCliError::NodeNotFound(locator.to_string()));
        }

        let mut node = NodeDescriptor::new(locator);
        if Self::locator_looks_sensitive(locator) {
            node.sensitive = true;
            node.text = Some("<hidden>".to_string());
        } else if locator.contains("text=") {
            node.text = Some("mock-text".to_string());
        }
        Ok(node)
    }

    fn locator_looks_sensitive(locator: &str) -> bool {
        let lowered = locator.to_ascii_lowercase();
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
    fn test_has_sensitive_nodes_fails_closed_until_real_scan_available() {
        let query = AtspiQuery;
        let app = crate::core::model::AppDescriptor::new("demo", 1);
        let has_sensitive = query
            .has_sensitive_nodes(&app)
            .expect("query should return result");
        assert!(has_sensitive);
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
