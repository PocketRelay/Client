use crate::{constants::HTTP_PORT, show_error};
use hyper::body::Body;
use hyper::service::service_fn;
use hyper::Response;
use hyper::{server::conn::Http, Request};
use std::convert::Infallible;
use std::{net::Ipv4Addr, process::exit};
use tokio::net::TcpListener;

pub async fn start_server() {
    // Initializing the underlying TCP listener
    let listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, HTTP_PORT)).await {
        Ok(value) => value,
        Err(err) => {
            let text = format!("Failed to start http: {}", err);
            show_error("Failed to start", &text);
            exit(1);
        }
    };

    // Accept incoming connections
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(value) => value,
            Err(_) => break,
        };

        tokio::task::spawn(async move {
            if let Err(err) = Http::new()
                .serve_connection(stream, service_fn(hello))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

// An async function that consumes a request, does nothing with it and returns a
// response.
async fn hello(req: Request<hyper::body::Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(req.into_body()))
}
