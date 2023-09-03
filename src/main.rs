mod db;
mod error;
mod fragment;
mod opts;
mod rate_limit;
mod routes;
mod service;
mod sortid;
mod util;

use std::net::{self, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::atomic::AtomicU64;

use anyhow::Context;
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use clap::Parser;
use hyper::http;
use hyper::server::conn::AddrIncoming;
use lettre::message::{Mailbox, MessageBuilder};
use lettre::{Address, SmtpTransport, Transport};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Default)]
struct State {
    count: AtomicU64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging()?;

    if let Ok(path) = dotenv::dotenv() {
        info!(path = %path.display(), "Loaded env file");
    }

    let opts = opts::Opts::parse();

    // send_email()?;

    let service = service::Service::new(&opts).await?;
    let app = axum::Router::new()
        .route("/", get(routes::home))
        .route("/item", post(routes::item_create))
        .route("/item/order", post(routes::item_order))
        .route("/item/:id/edit", get(routes::item_edit))
        .route("/favicon.ico", get(routes::favicon_ico))
        .route("/style.css", get(routes::style_css))
        .route("/script.js", get(routes::script_js))
        .route("/count", post(routes::count))
        .route("/user/:id", get(routes::get_user))
        .route("/post/:id", post(routes::save_post))
        .route("/post/:id/edit", get(routes::edit_post))
        .fallback(routes::not_found_404)
        .with_state(service.clone())
        .layer(middleware::from_fn_with_state(service, rate_limit))
        .layer(TraceLayer::new_for_http());

    let incoming = AddrIncoming::bind(&opts.listen.parse()?)?;
    info!("Listening on {}", incoming.local_addr());
    hyper::server::Server::builder(incoming)
        .serve(app.into_make_service())
        .await
        .context("Failed to start http server")?;

    Ok(())
}

async fn rate_limit<B>(
    axum::extract::State(service): axum::extract::State<service::Service>,
    req: http::Request<B>,
    next: middleware::Next<B>,
) -> Response {
    let peer_addr = req
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0);
    let peer_ip = peer_addr
        .map(|s| s.ip())
        .unwrap_or(net::IpAddr::V4(Ipv4Addr::UNSPECIFIED));

    if service.pre_rate_limiter.rate_limit(peer_ip) && service.rate_limiter.rate_limit(peer_ip) {
        routes::too_many_requests_429().await.into_response()
    } else {
        next.run(req).await
    }
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

#[allow(unused)]
fn send_email() -> anyhow::Result<()> {
    let smtp_hostname = std::env::var("SMTP_HOSTNAME")?;
    let smtp_port = std::env::var("SMTP_PORT")?;
    let smtp_username = std::env::var("SMTP_USER")?;
    let smtp_password = std::env::var("SMTP_PASSWORD")?;
    let smtp_to = std::env::var("SMTP_TO")?;
    let smtp_from = std::env::var("SMTP_FROM")?;

    let email = MessageBuilder::new()
        .to(Mailbox::new(None, Address::from_str(&smtp_to)?))
        .from(Mailbox::new(None, Address::from_str(&smtp_from)?))
        .subject("Test Email")
        .body("Hello from Rust!".to_owned())?;

    let mailer = SmtpTransport::relay(&smtp_hostname)?
        .port(FromStr::from_str(&smtp_port).context("Failed to parse port number")?)
        .credentials(lettre::transport::smtp::authentication::Credentials::new(
            smtp_username,
            smtp_password,
        ))
        .build();

    mailer
        .test_connection()
        .context("SMTP Connection test failed")?;

    mailer.send(&email).context("Failed to send email")?;

    Ok(())
}
