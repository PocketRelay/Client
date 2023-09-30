#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use config::read_config_file;
use hosts::HostEntryGuard;

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

    tokio::spawn(update::update());
    tokio::spawn(servers::start());

    // Initialize the UI
    ui::init(config);
}
