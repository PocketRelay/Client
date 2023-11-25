#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]
#![warn(unused_crate_dependencies)]

// Re-export the shared module as `core` for easier use
pub use pocket_relay_client_shared as core;

use crate::ui::show_error;
use config::read_config_file;
use core::{api::create_http_client, api::read_client_identity, reqwest};
use hosts::HostEntryGuard;
use log::error;
use std::path::Path;
use ui::show_confirm;

mod config;
mod constants;
mod hosts;
mod servers;
mod ui;
mod update;

fn main() {
    env_logger::builder()
        .filter_module("pocket_relay_client", log::LevelFilter::Debug)
        .init();

    // Attempt to apply the hosts file modification guard
    let _host_guard: Option<HostEntryGuard> = HostEntryGuard::apply();

    let config: Option<config::ClientConfig> = read_config_file();

    // Load the client identity
    let mut identity: Option<reqwest::Identity> = None;
    let identity_file = Path::new("pocket-relay-identity.p12");
    if identity_file.exists()
        && identity_file.is_file()
        && show_confirm(
            "Found client identity",
            "Detected client identity pocket-relay-identity.p12, would you like to use this identity?",
        )
    {
        identity = match read_client_identity(identity_file) {
            Ok(value) => Some(value),
            Err(err) => {
                error!("Failed to set client identity: {}", err);
                show_error("Failed to set client identity", &err.to_string());
                None
            }
        };
    }

    let client: reqwest::Client =
        create_http_client(identity).expect("Failed to create HTTP client");

    // Initialize the UI
    ui::init(config, client);
}
