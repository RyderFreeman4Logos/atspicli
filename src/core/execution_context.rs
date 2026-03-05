use crate::core::model::AppDescriptor;
use crate::error::{AtspiCliError, Result};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExecutionContext {
    pub target_app_name: Option<String>,
    pub target_pid: Option<u32>,
}

impl ExecutionContext {
    pub fn new(app_name: Option<String>, pid: Option<u32>) -> Self {
        Self {
            target_app_name: app_name,
            target_pid: pid,
        }
    }

    pub fn resolve_app(&self, apps: &[AppDescriptor]) -> Result<Option<AppDescriptor>> {
        if apps.is_empty() {
            return Err(AtspiCliError::AppResolution(
                "No accessible applications were discovered".to_string(),
            ));
        }

        match (&self.target_app_name, self.target_pid) {
            (Some(name), Some(pid)) => apps
                .iter()
                .find(|app| app.name == *name && app.pid == pid)
                .cloned()
                .map(Some)
                .ok_or_else(|| {
                    AtspiCliError::AppResolution(format!(
                        "No application matched --app '{}' and --pid '{}'",
                        name, pid
                    ))
                }),
            (Some(name), None) => {
                let matches: Vec<&AppDescriptor> =
                    apps.iter().filter(|app| app.name == *name).collect();
                if matches.is_empty() {
                    Err(AtspiCliError::AppResolution(format!(
                        "No application matched --app '{}'",
                        name
                    )))
                } else if matches.len() > 1 {
                    Err(AtspiCliError::AppResolution(format!(
                        "Multiple applications matched --app '{}'; provide --pid",
                        name
                    )))
                } else {
                    Ok(Some((*matches[0]).clone()))
                }
            }
            (None, Some(pid)) => apps
                .iter()
                .find(|app| app.pid == pid)
                .cloned()
                .map(Some)
                .ok_or_else(|| {
                    AtspiCliError::AppResolution(format!("No application matched --pid '{}'", pid))
                }),
            (None, None) => {
                if apps.len() == 1 {
                    Ok(Some(apps[0].clone()))
                } else {
                    Err(AtspiCliError::AppResolution(
                        "Multiple applications discovered; provide --app or --pid".to_string(),
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::execution_context::ExecutionContext;
    use crate::core::model::AppDescriptor;

    fn sample_apps() -> Vec<AppDescriptor> {
        vec![
            AppDescriptor::new("terminal", 100),
            AppDescriptor::new("editor", 200),
            AppDescriptor::new("terminal", 300),
        ]
    }

    #[test]
    fn test_resolve_app_with_pid() {
        let ctx = ExecutionContext::new(None, Some(200));
        let resolved = ctx.resolve_app(&sample_apps()).expect("resolve by pid");
        assert_eq!(resolved.expect("app").name, "editor");
    }

    #[test]
    fn test_resolve_app_with_name_and_pid() {
        let ctx = ExecutionContext::new(Some("terminal".to_string()), Some(300));
        let resolved = ctx.resolve_app(&sample_apps()).expect("resolve by pair");
        assert_eq!(resolved.expect("app").pid, 300);
    }

    #[test]
    fn test_resolve_app_with_ambiguous_name_fails() {
        let ctx = ExecutionContext::new(Some("terminal".to_string()), None);
        let err = ctx.resolve_app(&sample_apps()).expect_err("should fail");
        assert!(err
            .to_string()
            .contains("Multiple applications matched --app 'terminal'"));
    }

    #[test]
    fn test_resolve_app_without_selector_single_app() {
        let apps = vec![AppDescriptor::new("browser", 777)];
        let ctx = ExecutionContext::new(None, None);
        let resolved = ctx.resolve_app(&apps).expect("single app resolve");
        assert_eq!(resolved.expect("app").name, "browser");
    }
}
