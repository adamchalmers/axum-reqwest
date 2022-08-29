use axum::{
    body::Bytes,
    extract::{self, multipart::MultipartError, State},
    routing::post,
    Router,
};
use bytes::BytesMut;
use futures::Stream;
use reqwest::StatusCode;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // First, extract the program arguments.
    let mut args = std::env::args();

    // We don't need the 0th argument.
    let _program_invoked = args.next().unwrap();

    // Which port should this server run on?
    let listen_port: u16 = args
        .next()
        .expect("arg 1 must be a port to listen on")
        .parse()
        .expect("invalid port");

    // Is this server going to proxy bodies, or print them?
    // If proxy, 2nd argument is a port. Otherwise it's empty.
    let destination_port: Option<u16> = args
        .next()
        .map(|s| s.parse().expect("arg 2 must be a port, or an empty string"));

    let is_buffered = args.next().map(|s| match s.as_ref() {
        "--buffered" => true,
        "--streaming" => false,
        _ => panic!("must supply either --buffered or --streaming or nothing for 3rd argument"),
    });

    // Print the startup message.
    let listen_addr = SocketAddr::from(([127, 0, 0, 1], listen_port));
    let pid = std::process::id();
    if let Some(p) = &destination_port {
        println!("proxying {listen_port} -> {p} (PID {pid})")
    } else {
        println!("listening on {listen_port} (PID {pid})");
    }

    // Run the server.
    let builder = axum::Server::bind(&listen_addr);
    match (destination_port, is_buffered) {
        // Buffered proxy server
        (Some(dst_port), Some(true)) => builder
            .serve(
                Router::with_state(ProxyState {
                    dst_port,
                    client: reqwest::Client::new(),
                })
                .route("/", post(proxy_upload_buffered))
                .into_make_service(),
            )
            .await
            .unwrap(),
        // Streaming proxy server
        (Some(dst_port), Some(false)) => builder
            .serve(
                Router::with_state(ProxyState {
                    dst_port,
                    client: reqwest::Client::new(),
                })
                .route("/", post(proxy_upload_streaming))
                .into_make_service(),
            )
            .await
            .unwrap(),
        // Invalid
        (Some(_), None) => {
            println!(
                "Your 2nd argument says to start a proxy server, but your 3rd argument was neither --buffered nor --streaming."
            );
            std::process::exit(1)
        }
        // Print server
        (None, _) => builder
            .serve(
                Router::new()
                    .route("/", post(print_body))
                    .into_make_service(),
            )
            .await
            .unwrap(),
    }
}

/// State for the proxy server.
#[derive(Clone)]
struct ProxyState {
    dst_port: u16,
    client: reqwest::Client,
}

async fn proxy_upload_streaming(
    State(ProxyState { dst_port, client }): State<ProxyState>,
    incoming_body: extract::Multipart,
) -> Result<String, (StatusCode, String)> {
    let stream = MultipartStream(incoming_body).into_stream();
    let outgoing_body = reqwest::Body::wrap_stream(stream);
    client
        .post(url::Url::parse(&format!("http://127.0.0.1:{dst_port}/")).unwrap())
        .body(outgoing_body)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .map(|_| format!("All OK\n"))
}

async fn proxy_upload_buffered(
    State(ProxyState { dst_port, client }): State<ProxyState>,
    mut mp: extract::Multipart,
) -> Result<String, (StatusCode, String)> {
    // Buffer the whole body into memory.
    const BYTES_IN_DICT: usize = 2493109;
    let mut outgoing_body = BytesMut::with_capacity(BYTES_IN_DICT * 20);
    while let Some(field) = mp.next_field().await.unwrap() {
        outgoing_body.extend(field.bytes().await.unwrap())
    }
    // Send the body.
    client
        .post(url::Url::parse(&format!("http://127.0.0.1:{dst_port}/")).unwrap())
        .body(outgoing_body.freeze())
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .map(|_| format!("All OK\n"))
}

/// Wrapper for axum's Multipart type
struct MultipartStream(extract::Multipart);

impl MultipartStream {
    fn into_stream(mut self) -> impl Stream<Item = Result<Bytes, MultipartError>> {
        async_stream::stream! {
            while let Some(field) = self.0.next_field().await.unwrap() {
                for await value in field {
                    yield value;
                }
            }
        }
    }
}

/// Handler for the print server
async fn print_body(body: String) -> Result<String, (StatusCode, String)> {
    println!("Received request: {body}");
    Ok(String::new())
}
