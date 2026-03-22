use std::path::Path;
use std::process::Command;

use crate::error::{AtspiCliError, Result};

/// Detect which screenshot tool is available on the system.
fn detect_tool() -> Option<&'static str> {
    for tool in &["grim", "import", "gnome-screenshot", "scrot"] {
        if Command::new("which")
            .arg(tool)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(tool);
        }
    }
    None
}

/// Capture a region of the screen to the given output path.
pub(crate) fn capture_region(x: i32, y: i32, w: i32, h: i32, output: &Path) -> Result<()> {
    let output_str = output.to_str().ok_or_else(|| {
        AtspiCliError::InvalidArgument("Output path is not valid UTF-8".to_string())
    })?;

    let tool = detect_tool().ok_or_else(|| {
        AtspiCliError::Internal(
            "No screenshot tool found. Install one of: grim (Wayland), import (ImageMagick), gnome-screenshot, scrot".to_string(),
        )
    })?;

    let geometry = format!("{w}x{h}+{x}+{y}");
    let status = match tool {
        "grim" => Command::new("grim")
            .args(["-g", &format!("{x},{y} {w}x{h}"), output_str])
            .status(),
        "import" => Command::new("import")
            .args(["-window", "root", "-crop", &geometry, output_str])
            .status(),
        "scrot" => Command::new("scrot")
            .args(["-a", &format!("{x},{y},{w},{h}"), output_str])
            .status(),
        _ => Command::new(tool)
            .args(["-f", output_str, "-a", &geometry])
            .status(),
    };

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(AtspiCliError::Internal(format!(
            "Screenshot tool '{tool}' exited with status {s}"
        ))),
        Err(e) => Err(AtspiCliError::Io(e)),
    }
}

/// Capture the full screen or focused window to the given output path.
pub(crate) fn capture_window(output: &Path) -> Result<()> {
    let output_str = output.to_str().ok_or_else(|| {
        AtspiCliError::InvalidArgument("Output path is not valid UTF-8".to_string())
    })?;

    let tool = detect_tool().ok_or_else(|| {
        AtspiCliError::Internal(
            "No screenshot tool found. Install one of: grim (Wayland), import (ImageMagick), gnome-screenshot, scrot".to_string(),
        )
    })?;

    let status = match tool {
        "grim" => Command::new("grim").arg(output_str).status(),
        "import" => Command::new("import")
            .args(["-window", "root", output_str])
            .status(),
        "gnome-screenshot" => Command::new("gnome-screenshot")
            .args(["-f", output_str])
            .status(),
        "scrot" => Command::new("scrot").arg(output_str).status(),
        _ => Command::new(tool).arg(output_str).status(),
    };

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(AtspiCliError::Internal(format!(
            "Screenshot tool '{tool}' exited with status {s}"
        ))),
        Err(e) => Err(AtspiCliError::Io(e)),
    }
}
