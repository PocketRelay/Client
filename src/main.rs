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

// Native UI variant
#[cfg(feature = "native")]
use ui::native::init;
// Native UI variant
#[cfg(feature = "iced")]
use ui::iced::init;

fn main() {
    let _ = HostEntryGuard::apply();

    // Create tokio async runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");

    let config = runtime.block_on(read_config_file());

    runtime.spawn(update::update());

    // Start the servers
    runtime.spawn(servers::start());

    // Initialize the UI
    init(runtime, config);
}
