use clap::Parser;
use tracing_subscriber::EnvFilter;

use atspicli::adapters::atspi::AtspiBackend;
use atspicli::core::command::{CommandBackend, CommandExecutor};
use atspicli::core::execution_context::ExecutionContext;
use atspicli::error::Result;
use atspicli::ui_cli::parser::Cli;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .init();

    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(error.exit_code());
    }
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let context = ExecutionContext::new(cli.app, cli.pid);

    let backend = AtspiBackend::new();
    let executor = CommandExecutor::new(&backend as &dyn CommandBackend);
    let request: atspicli::core::command::CommandRequest = cli.command.into();

    let output = executor.execute(&context, &request)?;
    if let Some(rendered) = output.render() {
        println!("{rendered}");
    }
    Ok(())
}
