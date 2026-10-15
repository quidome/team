//! The team server: settings, database, API and commands (decision 0001).

pub mod api;
pub mod app;
pub mod auth;
pub mod changes;
pub mod clock;
pub mod config;
pub mod db;
pub mod error;
pub mod followed_teams;
pub mod games;
pub mod nightly;
pub mod players;
pub mod reading;
pub mod readings;
pub mod reasons;
pub mod source;
pub mod training_settings;
pub mod web;

pub use app::{AppState, app};
