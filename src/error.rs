use thiserror::Error;

#[derive(Debug, Error)]
pub enum AtspiCliError {
    #[error("AT-SPI error: {0}")]
    Atspi(String),

    #[error("DBus error: {0}")]
    DBus(String),

    #[error("Application resolution error: {0}")]
    AppResolution(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid locator: {0}")]
    InvalidLocator(String),

    #[error("Locator too complex: {0}")]
    LocatorTooComplex(String),

    #[error("Sensitive node policy violation: {0}")]
    SensitiveNodePolicy(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Timeout waiting for locator '{locator}' in {timeout_ms}ms")]
    Timeout { locator: String, timeout_ms: u64 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, AtspiCliError>;

impl AtspiCliError {
    pub const fn exit_code(&self) -> i32 {
        match self {
            AtspiCliError::Internal(_) => 1,
            AtspiCliError::Atspi(_) => 2,
            AtspiCliError::DBus(_) => 3,
            AtspiCliError::AppResolution(_) => 4,
            AtspiCliError::NodeNotFound(_) => 5,
            AtspiCliError::Io(_) => 6,
            AtspiCliError::InvalidLocator(_) => 7,
            AtspiCliError::LocatorTooComplex(_) => 7,
            AtspiCliError::SensitiveNodePolicy(_) => 8,
            AtspiCliError::InvalidArgument(_) => 9,
            AtspiCliError::Timeout { .. } => 10,
        }
    }
}

impl From<atspi::AtspiError> for AtspiCliError {
    fn from(value: atspi::AtspiError) -> Self {
        AtspiCliError::Atspi(value.to_string())
    }
}

impl From<zbus::Error> for AtspiCliError {
    fn from(value: zbus::Error) -> Self {
        AtspiCliError::DBus(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::AtspiCliError;

    #[test]
    fn test_exit_code_mapping_internal() {
        assert_eq!(AtspiCliError::Internal("x".into()).exit_code(), 1);
    }

    #[test]
    fn test_exit_code_mapping_locator_and_security() {
        assert_eq!(AtspiCliError::InvalidLocator("x".into()).exit_code(), 7);
        assert_eq!(AtspiCliError::LocatorTooComplex("x".into()).exit_code(), 7);
        assert_eq!(
            AtspiCliError::SensitiveNodePolicy("x".into()).exit_code(),
            8
        );
    }

    #[test]
    fn test_exit_code_mapping_argument_and_timeout() {
        assert_eq!(AtspiCliError::InvalidArgument("x".into()).exit_code(), 9);
        assert_eq!(
            AtspiCliError::Timeout {
                locator: "x".into(),
                timeout_ms: 1
            }
            .exit_code(),
            10
        );
    }
}
