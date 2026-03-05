use atspicli::adapters::mock::InMemoryBackend;
use atspicli::core::model::{AppDescriptor, NodeDescriptor};

pub fn build_backend() -> InMemoryBackend {
    let backend = InMemoryBackend::default();
    backend.add_app(AppDescriptor::new("demo-app", 1010));

    let mut root = NodeDescriptor::new("root");
    root.text = Some("Root".to_string());
    backend.add_node(root);

    let mut button = NodeDescriptor::new("button[text=Save]");
    button.text = Some("Save".to_string());
    backend.add_node(button);

    let mut input = NodeDescriptor::new("input[name=username]");
    input.text = Some("initial".to_string());
    backend.add_node(input);

    let mut secret = NodeDescriptor::new("input[name=password]");
    secret.text = Some("top-secret".to_string());
    secret.sensitive = true;
    backend.add_node(secret);

    backend.set_property("button[text=Save]", "role", "button");
    backend
}
