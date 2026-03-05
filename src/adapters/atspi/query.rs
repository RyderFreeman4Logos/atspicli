use crate::core::model::{AppDescriptor, NodeDescriptor};
use crate::error::{AtspiCliError, Result};

#[derive(Default)]
pub struct AtspiQuery;

impl AtspiQuery {
    pub fn list_applications_sync(&self) -> Result<Vec<AppDescriptor>> {
        #[cfg(debug_assertions)]
        if let Some(parsed) = Self::parse_fake_apps_from_env() {
            return Ok(parsed);
        }
        Ok(vec![AppDescriptor::new("default-app", std::process::id())])
    }

    pub fn has_sensitive_nodes(&self, _app: &AppDescriptor) -> Result<bool> {
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
        if locator.contains("password") || locator.contains("secret") {
            node.sensitive = true;
            node.text = Some("<hidden>".to_string());
        } else if locator.contains("text=") {
            node.text = Some("mock-text".to_string());
        }
        Ok(node)
    }

    pub async fn smoke_check_connection() -> Result<()> {
        let _ = zbus::Connection::session().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AtspiQuery;

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
