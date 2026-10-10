//! The team server: settings, database, API and commands (decision 0001).

pub mod api;
pub mod app;
pub mod auth;
pub mod clock;
pub mod config;
pub mod db;
pub mod error;
pub mod web;

pub use app::{AppState, app};
