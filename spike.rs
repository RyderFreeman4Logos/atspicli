use atspi::connection::AccessibilityConnection;
use atspi::proxy::accessible::AccessibleProxy;
use atspi::Role;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection = AccessibilityConnection::open().await?;
    let desktop = connection.get_desktop(0).await?;
    let child_count = desktop.child_count().await?;
    println!("Desktop has {} children", child_count);
    for i in 0..child_count {
        let child = desktop.get_child_at_index(i).await?;
        let role = child.get_role().await?;
        let name = child.name().await?;
        println!("Child {}: {} ({:?})", i, name, role);
    }
    Ok(())
}
