//! The `team` binary: `serve` runs the web server, `migrate` applies database migrations.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use clap::{Parser, Subcommand};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use team::auth::oidc::Oidc;
use team::clock::SystemClock;
use team::config::Config;
use team::{AppState, api, app, db};

#[derive(Parser)]
#[command(name = "team", about = "Team coordinator tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the web server.
    Serve,
    /// Apply database migrations that are not applied yet. Needs only DATABASE_URL.
    Migrate,
    /// Print the OpenAPI description of the API. Needs no settings.
    Openapi,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    match Cli::parse().command {
        Command::Serve => serve().await,
        Command::Migrate => migrate().await,
        Command::Openapi => {
            println!(
                "{}",
                api::openapi_json().context("writing the OpenAPI description")?
            );
            Ok(())
        }
    }
}

async fn serve() -> anyhow::Result<()> {
    let config = Config::from_env().context("reading settings")?;
    let pool = db::lazy_pool(&config.database_url).context("creating the database pool")?;
    let address = SocketAddr::new(config.host, config.port);
    let oidc = Oidc::new(
        &config.oidc_issuer_url,
        &config.oidc_client_id,
        &config.oidc_client_secret,
        &config.origin,
    )
    .context("setting up the login client")?;
    if config.dev_auth_bypass {
        tracing::warn!("DEV_AUTH_BYPASS is on: every request is the user Developer");
    }
    let state = AppState {
        pool: pool.clone(),
        config: Arc::new(config),
        oidc: Arc::new(oidc),
        clock: Arc::new(SystemClock),
    };

    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("listening on {address}"))?;
    tracing::info!(%address, "serving");
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serving")?;

    pool.close().await;
    tracing::info!("stopped");
    Ok(())
}

async fn migrate() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").context("setting DATABASE_URL is required")?;
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .context("connecting to the database")?;
    db::migrate(&pool).await.context("applying migrations")?;
    pool.close().await;
    tracing::info!("migrations are up to date");
    Ok(())
}

/// Resolves on Ctrl-C or SIGTERM (the stop signal from Kubernetes).
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "cannot listen for Ctrl-C");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "cannot listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("stopping");
}
