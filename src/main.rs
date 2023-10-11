mod db;
mod fragment;
mod opts;
mod rate_limit;
mod response;
mod routes;
mod service;
mod sortid;
mod util;

use anyhow::Context;
use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::service::Service;

fn main() -> anyhow::Result<()> {
    init_logging()?;

    if let Ok(path) = dotenv::dotenv() {
        info!(path = %path.display(), "Loaded env file");
    }

    let opts = opts::Opts::parse();

    let service = Service::new(opts.clone())?;

    let server = astra::Server::bind(opts.listen);

    info!("Listening on {}", server.local_addr()?);
    server
        .serve_clone(service)
        .context("Failed to start http server")?;

    Ok(())
}

fn init_logging() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // Print to stderr
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    Ok(())
}
