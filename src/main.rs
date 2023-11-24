#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use std::path::Path;

use config::read_config_file;
use hosts::HostEntryGuard;
use log::error;
use pocket_relay_client_shared::api::{create_http_client, read_client_identity};
use reqwest::Client;

use crate::ui::show_error;

mod config;
mod constants;
mod hosts;
mod servers;
mod ui;
mod update;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("pocket_relay_client", log::LevelFilter::Debug)
        .init();

    let _host_guard: HostEntryGuard = HostEntryGuard::apply();

    let config: Option<config::ClientConfig> = read_config_file().await;

    // Load the client identity
    let mut identity: Option<reqwest::Identity> = None;
    let identity_file = Path::new("pocket-relay-identity.p12");
    if identity_file.exists() && identity_file.is_file() {
        identity = match read_client_identity(identity_file).await {
            Ok(value) => Some(value),
            Err(err) => {
                error!("Failed to set client identity: {}", err);
                show_error("Failed to set client identity", &err.to_string());
                None
            }
        };
    }

    let client: Client = create_http_client(identity).expect("Failed to create HTTP client");

    tokio::spawn(update::update(client.clone()));

    // Initialize the UI
    ui::init(config, client);
}
