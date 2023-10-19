#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use config::{load_client_identity, read_config_file};
use constants::PR_USER_AGENT;
use hosts::HostEntryGuard;
use reqwest::Client;
use servers::tunnel;
use tokio::task::spawn_blocking;

mod api;
mod config;
mod constants;
mod hosts;
mod patch;
mod servers;
mod ui;
mod update;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("pocket_relay_client", log::LevelFilter::Debug)
        .init();

    let _host_guard = HostEntryGuard::apply();

    let config = read_config_file().await;

    let mut client_builder = Client::builder().user_agent(PR_USER_AGENT);

    if let Some(identity) = load_client_identity().await {
        client_builder = client_builder.identity(identity);
    }

    let client = client_builder.build().expect("Failed to build HTTP client");

    tokio::spawn(update::update(client.clone()));
    tokio::spawn(servers::start(client.clone()));
    tokio::spawn(spawn_blocking(|| tunnel::start_tunnel()));

    // Initialize the UI
    ui::init(config, client);
}
