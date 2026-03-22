use atspi::proxy::accessible::AccessibleProxy;
use atspi::Role;
use serde::Serialize;

use crate::core::model::NodeDescriptor;

/// A node in the accessibility tree, used for serialization and matching.
#[derive(Clone, Debug, Serialize)]
pub struct TreeNode {
    pub role: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub states: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TreeNode>,
    /// D-Bus bus name of the owning application (not serialized).
    #[serde(skip)]
    pub bus_name: String,
    /// D-Bus object path of this accessible node (not serialized).
    #[serde(skip)]
    pub object_path: String,
}

impl TreeNode {
    pub fn is_sensitive(&self) -> bool {
        self.states.iter().any(|s| s == "sensitive")
            || self.role == "password text"
            || super::query::AtspiQuery::locator_looks_sensitive_str(&self.name)
    }

    pub fn to_node_descriptor(&self, locator: &str) -> NodeDescriptor {
        let mut desc = NodeDescriptor::new(locator);
        desc.text = if self.name.is_empty() {
            None
        } else {
            Some(self.name.clone())
        };
        desc.sensitive = self.is_sensitive();
        desc.visible = self.states.iter().any(|s| s == "visible" || s == "showing");
        desc
    }
}

/// Walk the accessibility tree starting from the given proxy, up to `max_depth` levels.
/// A `max_depth` of -1 means unlimited.
///
/// `proxy_bus_name` and `proxy_obj_path` identify the D-Bus coordinates of the
/// given proxy so they can be stored in the resulting `TreeNode` for later use
/// when executing actions.
pub async fn walk_tree(
    conn: &zbus::Connection,
    proxy: &AccessibleProxy<'_>,
    max_depth: i32,
    current_depth: i32,
    proxy_bus_name: &str,
    proxy_obj_path: &str,
) -> std::result::Result<TreeNode, zbus::Error> {
    let role = proxy.get_role().await.unwrap_or(Role::Invalid);
    let name = proxy.name().await.unwrap_or_default();
    let description = proxy.description().await.ok().filter(|d| !d.is_empty());

    let state_set = proxy.get_state().await.unwrap_or_default();
    let states: Vec<String> = format_states(&state_set);

    let mut children = Vec::new();
    if max_depth == -1 || current_depth < max_depth {
        let child_count = proxy.child_count().await.unwrap_or(0);
        for i in 0..child_count {
            let child_accessible = match proxy.get_child_at_index(i).await {
                Ok(c) => c,
                Err(_) => continue,
            };
            let child_bus = child_accessible.name.to_string();
            let child_path = child_accessible.path.to_string();
            let child_proxy = match AccessibleProxy::builder(conn)
                .destination(child_bus.as_str())
                .and_then(|b| b.path(child_path.as_str()))
            {
                Ok(b) => match b.build().await {
                    Ok(p) => p,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };
            if let Ok(child_node) = Box::pin(walk_tree(
                conn,
                &child_proxy,
                max_depth,
                current_depth + 1,
                &child_bus,
                &child_path,
            ))
            .await
            {
                children.push(child_node);
            }
        }
    }

    Ok(TreeNode {
        role: format_role(role),
        name,
        description,
        states,
        children,
        bus_name: proxy_bus_name.to_string(),
        object_path: proxy_obj_path.to_string(),
    })
}

/// Search the tree for the first node matching the given locator.
/// Returns the matching `TreeNode` if found.
///
/// `app_bus_name` and `app_obj_path` identify the D-Bus coordinates of
/// `app_proxy` so they propagate into the returned `TreeNode`.
pub async fn find_node(
    conn: &zbus::Connection,
    app_proxy: &AccessibleProxy<'_>,
    locator: &str,
    app_bus_name: &str,
    app_obj_path: &str,
) -> std::result::Result<Option<TreeNode>, zbus::Error> {
    let segments = parse_locator_segments(locator);
    if segments.is_empty() {
        return Ok(None);
    }

    // Special case: "root" matches the app root directly
    if segments.len() == 1
        && segments[0].role.as_deref() == Some("root")
        && segments[0].text_exact.is_none()
        && segments[0].text_contains.is_none()
    {
        let tree = walk_tree(conn, app_proxy, 0, 0, app_bus_name, app_obj_path).await?;
        return Ok(Some(tree));
    }

    // Walk the tree and match segments
    let tree = walk_tree(conn, app_proxy, -1, 0, app_bus_name, app_obj_path).await?;
    Ok(match_segments(&tree, &segments, 0, false))
}

/// Check whether any node in the app's tree is sensitive.
pub async fn has_any_sensitive(
    conn: &zbus::Connection,
    app_proxy: &AccessibleProxy<'_>,
) -> std::result::Result<bool, zbus::Error> {
    check_sensitive_recursive(conn, app_proxy, 10).await
}

async fn check_sensitive_recursive(
    conn: &zbus::Connection,
    proxy: &AccessibleProxy<'_>,
    max_depth: i32,
) -> std::result::Result<bool, zbus::Error> {
    if max_depth <= 0 {
        return Ok(false);
    }

    let role = proxy.get_role().await.unwrap_or(Role::Invalid);
    if role == Role::PasswordText {
        return Ok(true);
    }

    let state_set = proxy.get_state().await.unwrap_or_default();
    let states = format_states(&state_set);
    if states.iter().any(|s| s == "sensitive") {
        return Ok(true);
    }

    let name = proxy.name().await.unwrap_or_default();
    if super::query::AtspiQuery::locator_looks_sensitive_str(&name) {
        return Ok(true);
    }

    let child_count = proxy.child_count().await.unwrap_or(0);
    for i in 0..child_count {
        let child_accessible = match proxy.get_child_at_index(i).await {
            Ok(c) => c,
            Err(_) => continue,
        };
        let child_proxy = match AccessibleProxy::builder(conn)
            .destination(child_accessible.name.as_str())
            .and_then(|b| b.path(child_accessible.path.as_ref()))
        {
            Ok(b) => match b.build().await {
                Ok(p) => p,
                Err(_) => continue,
            },
            Err(_) => continue,
        };
        if Box::pin(check_sensitive_recursive(conn, &child_proxy, max_depth - 1)).await? {
            return Ok(true);
        }
    }

    Ok(false)
}

// --- Locator parsing and matching ---

#[derive(Debug)]
struct LocatorSegment {
    role: Option<String>,
    text_exact: Option<String>,
    text_contains: Option<String>,
    is_descendant: bool, // true = >>, false = > (direct child)
}

impl LocatorSegment {
    fn matches_node(&self, node: &TreeNode) -> bool {
        if let Some(role) = &self.role {
            if role != "root" && !node.role.eq_ignore_ascii_case(role) {
                // Also try matching without spaces (e.g., "pushbutton" matches "push button")
                let compact_role: String =
                    node.role.chars().filter(|c| !c.is_whitespace()).collect();
                if !compact_role.eq_ignore_ascii_case(role) {
                    return false;
                }
            }
        }
        if let Some(exact) = &self.text_exact {
            if node.name != *exact {
                return false;
            }
        }
        if let Some(contains) = &self.text_contains {
            if !node.name.contains(contains.as_str()) {
                return false;
            }
        }
        true
    }
}

fn parse_locator_segments(locator: &str) -> Vec<LocatorSegment> {
    let trimmed = locator.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut remaining = trimmed;
    let mut is_first = true;

    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }

        let is_descendant;
        if is_first {
            is_descendant = true; // first segment is always a descendant search from root
            is_first = false;
        } else if remaining.starts_with(">>") {
            remaining = remaining[2..].trim_start();
            is_descendant = true;
        } else if remaining.starts_with('>') {
            remaining = remaining[1..].trim_start();
            is_descendant = false;
        } else {
            // implicit descendant
            is_descendant = true;
        }

        // Parse role name (up to '[', ':', '>', or space)
        let mut role_end = remaining.len();
        for (i, c) in remaining.char_indices() {
            if c == '[' || c == ':' || c == '>' || c == ' ' {
                role_end = i;
                break;
            }
        }
        let role_str = &remaining[..role_end];
        let role = if role_str.is_empty() {
            None
        } else {
            Some(role_str.to_string())
        };

        remaining = &remaining[role_end..];

        // Parse text selectors [text=X] or [text~=X]
        let mut text_exact = None;
        let mut text_contains = None;

        while remaining.starts_with('[') {
            if let Some(end_bracket) = remaining.find(']') {
                let inside = &remaining[1..end_bracket];
                if let Some(val) = inside.strip_prefix("text=") {
                    text_exact = Some(val.to_string());
                } else if let Some(val) = inside.strip_prefix("text~=") {
                    text_contains = Some(val.to_string());
                } else if let Some(val) = inside.strip_prefix("name=") {
                    text_exact = Some(val.to_string());
                }
                remaining = &remaining[end_bracket + 1..];
            } else {
                break;
            }
        }

        // Skip pseudo-selectors like :visible, :has(...)
        while remaining.starts_with(':') {
            if remaining.starts_with(":has(") {
                // Skip to matching closing paren
                let mut depth = 0;
                let mut end = remaining.len();
                for (i, c) in remaining.char_indices() {
                    match c {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                end = i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                remaining = &remaining[end..];
            } else if remaining.starts_with(":visible") {
                remaining = &remaining[":visible".len()..];
            } else {
                // Unknown pseudo-selector, skip to next space or end
                let skip_end = remaining
                    .find([' ', '>'])
                    .unwrap_or(remaining.len());
                remaining = &remaining[skip_end..];
            }
        }

        segments.push(LocatorSegment {
            role,
            text_exact,
            text_contains,
            is_descendant,
        });
    }

    segments
}

fn match_segments(
    node: &TreeNode,
    segments: &[LocatorSegment],
    seg_idx: usize,
    check_self: bool,
) -> Option<TreeNode> {
    if seg_idx >= segments.len() {
        return None;
    }

    let segment = &segments[seg_idx];
    let is_last = seg_idx == segments.len() - 1;

    // Check if current node matches the current segment
    if check_self && segment.matches_node(node) {
        if is_last {
            return Some(node.clone());
        }
        // Continue matching remaining segments against children
        let next_seg = &segments[seg_idx + 1];
        for child in &node.children {
            if next_seg.is_descendant {
                if let Some(found) = match_segments(child, segments, seg_idx + 1, true) {
                    return Some(found);
                }
            } else {
                // Direct child only
                if segments[seg_idx + 1].matches_node(child) {
                    if seg_idx + 1 == segments.len() - 1 {
                        return Some(child.clone());
                    }
                    if let Some(found) = match_segments(child, segments, seg_idx + 1, true) {
                        return Some(found);
                    }
                }
            }
        }
    }

    // For descendant search, recurse into children
    if segment.is_descendant || !check_self {
        for child in &node.children {
            if let Some(found) = match_segments(child, segments, seg_idx, true) {
                return Some(found);
            }
        }
    }

    None
}

fn format_role(role: Role) -> String {
    format!("{role:?}")
        .chars()
        .fold(String::new(), |mut acc, c| {
            if c.is_uppercase() && !acc.is_empty() {
                acc.push(' ');
            }
            acc.push(c.to_ascii_lowercase());
            acc
        })
}

fn format_states(state_set: &atspi::StateSet) -> Vec<String> {
    use atspi::State;
    let all_states = [
        State::Active,
        State::Armed,
        State::Busy,
        State::Checked,
        State::Collapsed,
        State::Defunct,
        State::Editable,
        State::Enabled,
        State::Expandable,
        State::Expanded,
        State::Focusable,
        State::Focused,
        State::HasTooltip,
        State::Horizontal,
        State::Iconified,
        State::Modal,
        State::MultiLine,
        State::Multiselectable,
        State::Opaque,
        State::Pressed,
        State::Resizable,
        State::Selectable,
        State::Selected,
        State::Sensitive,
        State::Showing,
        State::SingleLine,
        State::Stale,
        State::Transient,
        State::Vertical,
        State::Visible,
        State::ManagesDescendants,
        State::Indeterminate,
        State::Required,
        State::Truncated,
        State::Animated,
        State::InvalidEntry,
        State::SupportsAutocompletion,
        State::SelectableText,
        State::IsDefault,
        State::Visited,
        State::Checkable,
        State::HasPopup,
        State::ReadOnly,
    ];

    all_states
        .iter()
        .filter(|s| state_set.contains(**s))
        .map(|s| format!("{s:?}").to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_locator_simple_role() {
        let segments = parse_locator_segments("button");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].role.as_deref(), Some("button"));
    }

    #[test]
    fn test_parse_locator_with_text() {
        let segments = parse_locator_segments("button[text=Save]");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].role.as_deref(), Some("button"));
        assert_eq!(segments[0].text_exact.as_deref(), Some("Save"));
    }

    #[test]
    fn test_parse_locator_descendant_chain() {
        let segments = parse_locator_segments("window >> button[text=Ok]");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].role.as_deref(), Some("window"));
        assert_eq!(segments[1].role.as_deref(), Some("button"));
        assert_eq!(segments[1].text_exact.as_deref(), Some("Ok"));
        assert!(segments[1].is_descendant);
    }

    #[test]
    fn test_parse_locator_child_chain() {
        let segments = parse_locator_segments("window > toolbar > button");
        assert_eq!(segments.len(), 3);
        assert!(!segments[1].is_descendant);
        assert!(!segments[2].is_descendant);
    }

    #[test]
    fn test_parse_locator_text_contains() {
        let segments = parse_locator_segments("item[text~=Task]");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text_contains.as_deref(), Some("Task"));
    }

    #[test]
    fn test_format_role() {
        assert_eq!(format_role(Role::PushButton), "push button");
        assert_eq!(format_role(Role::Frame), "frame");
        assert_eq!(format_role(Role::PasswordText), "password text");
    }

    #[test]
    fn test_segment_matches_node() {
        let node = TreeNode {
            role: "push button".into(),
            name: "Save".into(),
            description: None,
            states: vec![],
            children: vec![],
            bus_name: String::new(),
            object_path: String::new(),
        };
        let seg = LocatorSegment {
            role: Some("pushbutton".into()),
            text_exact: Some("Save".into()),
            text_contains: None,
            is_descendant: true,
        };
        assert!(seg.matches_node(&node));

        // Exact role name match
        let seg_exact = LocatorSegment {
            role: Some("push button".into()),
            text_exact: None,
            text_contains: None,
            is_descendant: true,
        };
        assert!(seg_exact.matches_node(&node));
    }

    #[test]
    fn test_tree_node_is_sensitive() {
        let mut node = TreeNode {
            role: "text".into(),
            name: "regular".into(),
            description: None,
            states: vec![],
            children: vec![],
            bus_name: String::new(),
            object_path: String::new(),
        };
        assert!(!node.is_sensitive());

        node.role = "password text".into();
        assert!(node.is_sensitive());

        node.role = "text".into();
        node.states = vec!["sensitive".into()];
        assert!(node.is_sensitive());
    }
}
