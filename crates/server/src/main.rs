//! The `team` binary: `serve` runs the web server and the nightly reading, `migrate` applies
//! database migrations.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use clap::{Parser, Subcommand};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use team::auth::oidc::Oidc;
#[cfg(debug_assertions)]
use team::clock::ShiftedClock;
use team::clock::{Clock, SystemClock};
use team::config::{self, Config};
use team::db::StoreError;
use team::source::Source;
use team::web::Web;
use team::{AppState, api, app, db, nightly, players};

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
    /// Apply database migrations that are not applied yet. Needs DATABASE_URL and PLAYER_NAME_KEY.
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
    let config_web_dir = config.web_dir.clone();
    let oidc = Oidc::new(
        &config.oidc_issuer_url,
        &config.oidc_client_id,
        &config.oidc_client_secret,
        &config.origin,
    )
    .context("setting up the login client")?;
    let source = Source::new(config.game_list_url.clone(), &config.origin)
        .context("setting up the game list reader")?;
    #[cfg(debug_assertions)]
    if config.dev_auth_bypass {
        tracing::warn!("DEV_AUTH_BYPASS is on: every request is the user Developer");
    }
    #[cfg(debug_assertions)]
    let clock: Arc<dyn Clock> = match config.dev_now {
        Some(start) => {
            tracing::warn!(%start, "DEV_NOW is set: the clock is shifted");
            Arc::new(ShiftedClock::starting_at(start))
        }
        None => Arc::new(SystemClock),
    };
    #[cfg(not(debug_assertions))]
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let source = Arc::new(source);
    let state = AppState {
        pool: pool.clone(),
        config: Arc::new(config),
        oidc: Arc::new(oidc),
        clock: clock.clone(),
        web: Arc::new(Web::load(&config_web_dir)),
        source: source.clone(),
    };

    // A wrong key stops the server before it shows a page. A database that cannot be reached does
    // not: the server starts, and readiness reports the database (`team migrate` in the init
    // container has checked the key already).
    match players::check_key(&state.pool, &state.config.player_name_key).await {
        Ok(()) => {}
        Err(StoreError::Database(error)) => {
            tracing::warn!(%error, "cannot check the player name key now: the database cannot be read");
        }
        Err(error) => {
            return Err(error).context("the player name key cannot read the stored names");
        }
    }

    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("listening on {address}"))?;
    tracing::info!(%address, "serving");
    let nightly = nightly::spawn(pool.clone(), source, clock);
    let served = axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serving");

    // Stopped before the pool closes, which would wait for a reading that holds a connection; a
    // reading stopped halfway rolls back.
    nightly.abort();
    served?;
    pool.close().await;
    tracing::info!("stopped");
    Ok(())
}

async fn migrate() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").context("setting DATABASE_URL is required")?;
    let player_name_key = config::player_name_key(std::env::var("PLAYER_NAME_KEY").ok())
        .context("reading settings")?;
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .context("connecting to the database")?;
    db::migrate(&pool).await.context("applying migrations")?;
    players::check_key(&pool, &player_name_key)
        .await
        .context("the player name key cannot read the stored names")?;
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
