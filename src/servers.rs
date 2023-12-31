use crate::{
    core::{ctx::ClientContext, servers::*},
    ui::show_error,
};
use log::error;
use std::{future::Future, sync::Arc};

/// Starts all the servers in their own tasks
///
/// ## Arguments
/// * `ctx` - The client context
pub fn start_all_servers(ctx: Arc<ClientContext>) {
    // Stop existing servers and tasks if they are running
    stop_server_tasks();

    // Create server tasks
    let redirector = redirector::start_redirector_server();
    let blaze = blaze::start_blaze_server(ctx.clone());
    let http = http::start_http_server(ctx.clone());
    let tunnel = tunnel::start_tunnel_server(ctx.clone());
    let qos = qos::start_qos_server();
    let telemetry = telemetry::start_telemetry_server(ctx);

    // Spawn server tasks
    run_server(redirector, "redirector");
    run_server(blaze, "blaze");
    run_server(http, "http");
    run_server(tunnel, "tunnel");
    run_server(qos, "qos");
    run_server(telemetry, "telemetry");
}

/// Runs the provided server `future` in a background task displaying
/// and logging any errors if they occur
#[inline]
pub fn run_server<F>(future: F, name: &'static str)
where
    F: Future<Output = std::io::Result<()>> + Send + 'static,
{
    spawn_server_task(async move {
        if let Err(err) = future.await {
            show_error(&format!("Failed to start {name} server"), &err.to_string());
            error!("Failed to start {name} server: {err}");
        }
    });
}
