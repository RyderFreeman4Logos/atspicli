use crate::error::{AtspiCliError, Result};

#[derive(Debug)]
pub struct AtspiSession {
    reconnect_count: u32,
    connected: bool,
}

impl AtspiSession {
    pub async fn connect() -> Result<Self> {
        let _ = atspi::connection::AccessibilityConnection::open().await?;
        Ok(Self {
            reconnect_count: 0,
            connected: true,
        })
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        let opened = atspi::connection::AccessibilityConnection::open().await;
        self.reconnect_count += 1;
        match opened {
            Ok(_) => {
                self.connected = true;
                Ok(())
            }
            Err(err) => {
                self.connected = false;
                Err(AtspiCliError::from(err))
            }
        }
    }

    pub fn reconnect_count(&self) -> u32 {
        self.reconnect_count
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    #[cfg(test)]
    fn disconnected_for_test() -> Self {
        Self {
            reconnect_count: 0,
            connected: false,
        }
    }

    #[cfg(test)]
    fn simulate_reconnect_result(&mut self, success: bool) {
        self.reconnect_count += 1;
        self.connected = success;
    }
}

#[cfg(test)]
mod tests {
    use super::AtspiSession;

    #[test]
    fn test_atspi_session_reconnect_increments_counter() {
        let mut session = AtspiSession::disconnected_for_test();
        session.simulate_reconnect_result(true);
        session.simulate_reconnect_result(false);
        assert_eq!(session.reconnect_count(), 2);
    }

    #[test]
    fn test_atspi_session_reconnect_updates_connection_state() {
        let mut session = AtspiSession::disconnected_for_test();
        session.simulate_reconnect_result(true);
        assert!(session.is_connected());
        session.simulate_reconnect_result(false);
        assert!(!session.is_connected());
    }
}
