//! # BootForge Core
//!
//! Core library for BootForge — the universal boot media manager.
//!
//! This crate provides shared types, configuration management, error definitions,
//! and event system used by all other BootForge components.

pub mod config;
pub mod error;
pub mod events;
pub mod types;

pub use config::AppConfig;
pub use error::{BootForgeError, Result};
pub use types::*;

/// BootForge application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "BootForge";

/// Application tagline
pub const TAGLINE: &str = "One Drive. Every OS.";
