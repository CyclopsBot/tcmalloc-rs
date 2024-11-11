#![deny(clippy::pedantic)]

use app::{CiCommands, Cli, Commands};
use clap::Parser;
use session::CyclopsSession;
use starbase::{App, MainResult};
use std::process::ExitCode;
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[global_allocator]
static GLOBAL: tcmalloc::TcMalloc = tcmalloc::TcMalloc;

#[tokio::main]
async fn main() -> MainResult {
    let app = App::default();
    let cli = Cli::parse();
    app.setup_diagnostics();
    let fmt_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(LevelFilter::INFO)
        .init();

    let session = CyclopsSession::new(cli);

    let run = app
        .run(session, |session| async move {
            match session.cli.command.clone() {
                Commands::Init(args) => commands::init(&args),
                Commands::Ci { command } => match command {
                    CiCommands::Annotate(args) => commands::ci::annotate(&args),
                    CiCommands::Healthcheck(args) => commands::ci::healthcheck(&args),
                },
            }
            Ok(None)
        })
        .await?;

    Ok(ExitCode::from(run))
}
