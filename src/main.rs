#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]
#![warn(unused_crate_dependencies)]

use crate::ui::show_error;
use config::read_config_file;
use core::{api::create_http_client, api::read_client_identity, reqwest};
use hosts::HostEntryGuard;
use log::error;
use pocket_relay_client_shared as core;
use std::path::Path;
use ui::show_confirm;

mod config;
mod hosts;
mod servers;
mod ui;
mod update;

/// Application crate version string
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    // Initialize logging
    env_logger::builder()
        .filter_module("pocket_relay_client", log::LevelFilter::Debug)
        .init();

    // Attempt to apply the hosts file modification guard
    let _host_guard: Option<HostEntryGuard> = HostEntryGuard::apply();

    // Load the config file
    let config: Option<config::ClientConfig> = read_config_file();

    // Load the client identity
    let identity: Option<reqwest::Identity> = load_identity();

    // Create the internal HTTP client
    let client: reqwest::Client =
        create_http_client(identity).expect("Failed to create HTTP client");

    // Initialize the UI
    ui::init(config, client);
}

/// Attempts to load an identity file if one is present
fn load_identity() -> Option<reqwest::Identity> {
    // Load the client identity
    let identity_file = Path::new("pocket-relay-identity.p12");

    // Handle no identity or user declining identity
    if !identity_file.exists() || !show_confirm(
        "Found client identity",
        "Detected client identity pocket-relay-identity.p12, would you like to use this identity?",
    ) {
        return None;
    }

    // Read the client identity
    match read_client_identity(identity_file) {
        Ok(value) => Some(value),
        Err(err) => {
            error!("Failed to set client identity: {}", err);
            show_error("Failed to set client identity", &err.to_string());
            None
        }
    }
}
