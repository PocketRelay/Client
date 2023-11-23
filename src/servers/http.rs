use crate::api::TARGET;
use crate::constants::HTTP_PORT;
use crate::ui::show_error;
use hyper::body::Body;
use hyper::service::service_fn;
use hyper::{server::conn::Http, Request};
use hyper::{Response, StatusCode};
use log::{debug, error};
use reqwest::Client;
use std::convert::Infallible;
use std::{net::Ipv4Addr, process::exit};
use tokio::net::TcpListener;

pub async fn start_server(http_client: Client) {
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

        let http_client = http_client.clone();

        tokio::task::spawn(async move {
            if let Err(err) = Http::new()
                .serve_connection(
                    stream,
                    service_fn(move |req| proxy_http(req, http_client.clone())),
                )
                .await
            {
                error!("Failed to serve http connection: {:?}", err);
            }
        });
    }
}

async fn proxy_http(req: Request<Body>, http_client: Client) -> Result<Response<Body>, Infallible> {
    // Get the path and query segement from the URL
    let path = req
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or_default();

    // Attempt to create the proxy target URL
    let target_url = {
        let target_guard = TARGET.read().await;

        let target = match target_guard.as_ref() {
            Some(value) => value,
            None => {
                let mut error_response = Response::default();
                *error_response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
                return Ok(error_response);
            }
        };

        debug!("{}", path);

        match target.url.join(path) {
            Ok(value) => value,
            Err(_) => {
                // Failed to create a path
                let mut error_response = Response::default();
                *error_response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
                return Ok(error_response);
            }
        }
    };

    let response = match proxy_request(http_client, target_url).await {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to proxy HTTP request: {}", err);

            let mut error_response = Response::default();
            *error_response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(error_response);
        }
    };

    Ok(response)
}

/// Makes the proxy request to the target url provided, creating
/// a response on success or providing an error.
async fn proxy_request(
    http_client: Client,
    target_url: url::Url,
) -> Result<Response<Body>, reqwest::Error> {
    let response = http_client.get(target_url).send().await?;

    // Extract response status and headers before its consumed to load the body
    let status = response.status();
    let headers = response.headers().clone();

    let body = response.bytes().await?;

    // Create new response from the proxy response
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    *response.headers_mut() = headers;

    Ok(response)
}
