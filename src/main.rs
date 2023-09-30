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

fn main() {
    env_logger::builder()
        .filter_module("pocket_relay_client", log::LevelFilter::Debug)
        .init();

    let _host_guard = HostEntryGuard::apply();

    // Create tokio async runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");

    let config = runtime.block_on(read_config_file());

    runtime.spawn(update::update());
    runtime.spawn(servers::start());

    // Initialize the UI
    ui::init(runtime, config);
}
