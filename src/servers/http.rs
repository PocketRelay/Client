use crate::api::TARGET;
use crate::constants::HTTP_PORT;
use crate::ui::show_error;
use hyper::body::Body;
use hyper::service::service_fn;
use hyper::{server::conn::Http, Request};
use hyper::{Response, StatusCode};
use reqwest::Client;
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
                .serve_connection(stream, service_fn(proxy_http))
                .await
            {
                eprintln!("Failed to serve http connection: {:?}", err);
            }
        });
    }
}

async fn proxy_http(req: Request<hyper::body::Body>) -> Result<Response<Body>, Infallible> {
    let path = req
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or_default();

    let target_url = {
        let target_guard = TARGET.read().await;
        let target = match target_guard.as_ref() {
            Some(value) => value,
            None => {
                let mut error_response = Response::new(hyper::Body::empty());
                *error_response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
                return Ok(error_response);
            }
        };

        format!(
            "{}://{}:{}{}",
            target.scheme, target.host, target.port, path
        )
    };

    let client = Client::new();
    let proxy_response = match client.get(target_url).send().await {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to send HTTP request: {:?}", err);
            let mut error_response = Response::new(hyper::Body::empty());
            *error_response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(error_response);
        }
    };
    let status = proxy_response.status();
    let headers = proxy_response.headers().clone();

    let body = match proxy_response.bytes().await {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to read HTTP response body: {}", err);
            let mut error_response = Response::new(hyper::Body::empty());
            *error_response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(error_response);
        }
    };

    let mut response = Response::new(hyper::body::Body::from(body));
    *response.status_mut() = status;
    *response.headers_mut() = headers;

    Ok(response)
}
