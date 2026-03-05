#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppDescriptor {
    pub name: String,
    pub pid: u32,
}

impl AppDescriptor {
    pub fn new(name: impl Into<String>, pid: u32) -> Self {
        Self {
            name: name.into(),
            pid,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeDescriptor {
    pub locator: String,
    pub text: Option<String>,
    pub visible: bool,
    pub sensitive: bool,
}

impl NodeDescriptor {
    pub fn new(locator: impl Into<String>) -> Self {
        Self {
            locator: locator.into(),
            text: None,
            visible: true,
            sensitive: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirection {
    pub fn parse(input: &str) -> Option<Self> {
        match input.to_ascii_lowercase().as_str() {
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }
}
