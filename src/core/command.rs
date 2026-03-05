use std::path::Path;
use std::time::Duration;

use tracing::debug;

use crate::core::execution_context::ExecutionContext;
use crate::core::locator::validate_locator;
use crate::core::model::{AppDescriptor, NodeDescriptor, ScrollDirection};
use crate::core::redaction::redact_sensitive;
use crate::error::{AtspiCliError, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandRequest {
    Snapshot {
        locator: String,
    },
    Click {
        locator: String,
    },
    Dblclick {
        locator: String,
    },
    Input {
        locator: String,
        text: String,
    },
    Fill {
        locator: String,
        text: String,
    },
    Press {
        key: String,
    },
    Hover {
        locator: String,
    },
    Focus {
        locator: String,
    },
    ScrollTo {
        locator: String,
    },
    Scroll {
        direction: String,
        amount: u32,
    },
    Screenshot {
        locator: Option<String>,
        output: String,
    },
    Wait {
        locator: String,
        timeout_secs: u32,
    },
    Get {
        locator: String,
        property: String,
    },
    ListApps,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandOutput {
    Empty,
    Text(String),
    AppList(Vec<AppDescriptor>),
}

impl CommandOutput {
    pub fn render(&self) -> Option<String> {
        match self {
            CommandOutput::Empty => None,
            CommandOutput::Text(value) => Some(value.clone()),
            CommandOutput::AppList(apps) => Some(
                apps.iter()
                    .map(|app| format!("{} ({})", app.name, app.pid))
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
        }
    }
}

pub trait CommandBackend {
    fn list_apps(&self) -> Result<Vec<AppDescriptor>>;
    fn read_node(&self, locator: &str) -> Result<NodeDescriptor>;
    fn snapshot(&self, locator: &str) -> Result<String>;
    fn get_property(&self, locator: &str, property: &str) -> Result<String>;
    fn wait_for(&self, locator: &str, timeout: Duration) -> Result<()>;
    fn click(&self, locator: &str, times: u8) -> Result<()>;
    fn hover(&self, locator: &str) -> Result<()>;
    fn focus(&self, locator: &str) -> Result<()>;
    fn input_text(&self, locator: &str, text: &str, clear_first: bool) -> Result<()>;
    fn press_key(&self, key: &str) -> Result<()>;
    fn scroll_to(&self, locator: &str) -> Result<()>;
    fn scroll(&self, direction: ScrollDirection, amount: u32) -> Result<()>;
    fn screenshot(&self, locator: Option<&str>, output: &Path) -> Result<()>;
}

pub struct CommandExecutor<'a> {
    backend: &'a dyn CommandBackend,
}

impl<'a> CommandExecutor<'a> {
    pub fn new(backend: &'a dyn CommandBackend) -> Self {
        Self { backend }
    }

    pub fn execute(
        &self,
        context: &ExecutionContext,
        request: &CommandRequest,
    ) -> Result<CommandOutput> {
        if matches!(request, CommandRequest::ListApps) {
            return Ok(CommandOutput::AppList(self.backend.list_apps()?));
        }

        let apps = self.backend.list_apps()?;
        let _resolved_app = context.resolve_app(&apps)?;

        match request {
            CommandRequest::Snapshot { locator } => {
                self.validate_and_check_sensitive(locator)?;
                let snapshot = self.backend.snapshot(locator)?;
                Ok(CommandOutput::Text(snapshot))
            }
            CommandRequest::Click { locator } => {
                self.validate_locator(locator)?;
                self.backend.click(locator, 1)?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Dblclick { locator } => {
                self.validate_locator(locator)?;
                self.backend.click(locator, 2)?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Input { locator, text } => {
                self.validate_locator(locator)?;
                if let Err(err) = self.backend.focus(locator) {
                    debug!(
                        "focus failed before input, fallback to direct input: {}",
                        redact_sensitive(&err.to_string())
                    );
                }
                debug!("input text: {}", redact_sensitive(text));
                self.backend.input_text(locator, text, false)?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Fill { locator, text } => {
                self.validate_locator(locator)?;
                debug!("fill text: {}", redact_sensitive(text));
                self.backend.input_text(locator, text, true)?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Press { key } => {
                if key.trim().is_empty() {
                    return Err(AtspiCliError::InvalidArgument(
                        "Key cannot be empty".to_string(),
                    ));
                }
                self.backend.press_key(key)?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Hover { locator } => {
                self.validate_locator(locator)?;
                self.backend.hover(locator)?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Focus { locator } => {
                self.validate_locator(locator)?;
                self.backend.focus(locator)?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::ScrollTo { locator } => {
                self.validate_locator(locator)?;
                self.backend.scroll_to(locator)?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Scroll { direction, amount } => {
                let parsed = ScrollDirection::parse(direction).ok_or_else(|| {
                    AtspiCliError::InvalidArgument(format!(
                        "Unsupported scroll direction '{direction}'"
                    ))
                })?;
                self.backend.scroll(parsed, *amount)?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Screenshot { locator, output } => {
                if let Some(loc) = locator {
                    self.validate_and_check_sensitive(loc)?;
                }
                self.backend
                    .screenshot(locator.as_deref(), Path::new(output))?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Wait {
                locator,
                timeout_secs,
            } => {
                self.validate_locator(locator)?;
                self.backend
                    .wait_for(locator, Duration::from_secs(u64::from(*timeout_secs)))?;
                Ok(CommandOutput::Empty)
            }
            CommandRequest::Get { locator, property } => {
                self.validate_and_check_sensitive(locator)?;
                let value = self.backend.get_property(locator, property)?;
                Ok(CommandOutput::Text(value))
            }
            CommandRequest::ListApps => unreachable!(),
        }
    }

    fn validate_locator(&self, locator: &str) -> Result<()> {
        validate_locator(locator)
    }

    fn validate_and_check_sensitive(&self, locator: &str) -> Result<()> {
        validate_locator(locator)?;
        let node = self.backend.read_node(locator)?;
        if node.sensitive {
            return Err(AtspiCliError::SensitiveNodePolicy(format!(
                "Locator '{locator}' is sensitive and cannot be read or captured"
            )));
        }
        Ok(())
    }
}
