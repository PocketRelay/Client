use tokio::join;

pub mod main;
pub mod qos;
pub mod redirector;
pub mod telemetry;

/// Starts and waits for all the servers
pub async fn start() {
    join!(
        main::start_server(),
        qos::start_server(),
        redirector::start_server(),
        telemetry::start_server()
    );
}
