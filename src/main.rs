#![doc = include_str!("../README.md")]
// Clippy lints
#![warn(clippy::large_stack_arrays)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::panic)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::unreachable)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::allow_attributes_without_reason)]
#![warn(clippy::cognitive_complexity)]

use crate::config::Config;
use crate::error::Error;
use crate::rtsp::RtspClient;
use ehttpd::http::{Response, ResponseExt};
use ehttpd::Server;
use std::convert::Infallible;
use std::process;

mod config;
mod error;
mod hls;
mod rtsp;

/// The rtsp2hls app runloop
fn rtsp2hls(config: Config) -> Result<Infallible, Error> {
    // Initialize the RTSP client
    let rtsp_client = RtspClient::new(&config.RTSP2HLS_SOURCE, &config.RTSP2HLS_TEMPDIR);
    rtsp_client.spawn()?.detach();

    // Initialize HTTP server with connection callback
    let hls_server_listen = config.RTSP2HLS_LISTEN;
    let hls_server = Server::with_request_response(config.RTSP2HLS_MAXCONN, move |request| {
        match (request.method.as_ref(), request.target.as_ref()) {
            (b"GET" | b"HEAD", target) if target.ends_with(b".ts") => hls::get_fragment(&request, &config),
            (b"GET" | b"HEAD", b"/") => Response::new_307_temporaryredirect(b"/index.m3u8"),
            (b"GET" | b"HEAD", b"/index.m3u8") => hls::get_index(&request, &config),
            (b"GET" | b"HEAD", _) => Response::new_404_notfound(),
            (_, _) => Response::new_405_methodnotallowed(),
        }
    });

    // Start and monitor the HLS server task
    let Err(e) = hls_server.accept(hls_server_listen);
    Err(error!(with: e, "server task failed"))
}

pub fn main() {
    // Load config and enter server runloop
    let Err(e) = Config::from_env().and_then(rtsp2hls);
    e.log_to_stderr();
    process::exit(1);
}
