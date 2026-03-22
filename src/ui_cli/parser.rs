use crate::core::command::CommandRequest;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "atspicli",
    version,
    about = "Rust atspicli aligned with axcli.rs"
)]
pub struct Cli {
    /// Target application name
    #[arg(long)]
    pub app: Option<String>,

    /// Target process ID
    #[arg(long)]
    pub pid: Option<u32>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Take a snapshot of the UI tree
    Snapshot {
        /// Locator for the root node of the snapshot
        #[arg(default_value = "root")]
        locator: String,
        /// Maximum traversal depth (-1 for unlimited)
        #[arg(long, default_value = "3")]
        depth: i32,
    },
    /// Click an element
    Click {
        /// Locator for the element
        locator: String,
    },
    /// Double click an element
    Dblclick {
        /// Locator for the element
        locator: String,
    },
    /// Input text to an element
    Input {
        /// Locator for the element
        locator: String,
        /// Text to input
        text: String,
    },
    /// Fill text to an element (clear first)
    Fill {
        /// Locator for the element
        locator: String,
        /// Text to fill
        text: String,
    },
    /// Press a key
    Press {
        /// Key name or combination
        key: String,
    },
    /// Hover over an element
    Hover {
        /// Locator for the element
        locator: String,
    },
    /// Focus an element
    Focus {
        /// Locator for the element
        locator: String,
    },
    /// Scroll to an element
    ScrollTo {
        /// Locator for the element
        locator: String,
    },
    /// Scroll in a direction
    Scroll {
        /// Direction: up, down, left, right
        direction: String,
        /// Amount to scroll
        amount: u32,
    },
    /// Take a screenshot
    Screenshot {
        /// Locator for the element (optional, defaults to whole window)
        locator: Option<String>,
        /// Output file path (auto-generated if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Wait for an element or condition
    Wait {
        /// Locator for the element
        locator: String,
        /// Timeout in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u32,
    },
    /// Get property of an element
    Get {
        /// Locator for the element
        locator: String,
        /// Property name
        property: String,
    },
    /// List running accessible apps
    ListApps,
}

impl From<Commands> for CommandRequest {
    fn from(value: Commands) -> Self {
        match value {
            Commands::Snapshot { locator, depth } => CommandRequest::Snapshot { locator, depth },
            Commands::Click { locator } => CommandRequest::Click { locator },
            Commands::Dblclick { locator } => CommandRequest::Dblclick { locator },
            Commands::Input { locator, text } => CommandRequest::Input { locator, text },
            Commands::Fill { locator, text } => CommandRequest::Fill { locator, text },
            Commands::Press { key } => CommandRequest::Press { key },
            Commands::Hover { locator } => CommandRequest::Hover { locator },
            Commands::Focus { locator } => CommandRequest::Focus { locator },
            Commands::ScrollTo { locator } => CommandRequest::ScrollTo { locator },
            Commands::Scroll { direction, amount } => CommandRequest::Scroll { direction, amount },
            Commands::Screenshot { locator, output } => {
                let resolved_output = output.unwrap_or_else(|| {
                    use std::time::SystemTime;
                    let secs = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    format!("screenshot-{secs}.png")
                });
                CommandRequest::Screenshot {
                    locator,
                    output: resolved_output,
                }
            }
            Commands::Wait { locator, timeout } => CommandRequest::Wait {
                locator,
                timeout_secs: timeout,
            },
            Commands::Get { locator, property } => CommandRequest::Get { locator, property },
            Commands::ListApps => CommandRequest::ListApps,
        }
    }
}
