//! HLS request handlers to serve a filesystem-backed HLS stream

use crate::config::Config;
use ehttpd::http::{Request, Response, ResponseExt};
use std::fs::File;

/// Handles a GET request for `/index.m3u8`
pub fn get_index(request: &Request, config: &Config) -> Response {
    // Assert request target as this route is fixed
    assert_eq!(request.target, b"/index.m3u8", "invalid route");

    // Open the index file
    let path = config.RTSP2HLS_TEMPDIR.join("index.m3u8");
    let Ok(file) = File::open(path) else {
        // We cannot open the index file
        return Response::new_404_notfound();
    };

    // Assemble response
    let mut response = Response::new_200_ok();
    let Ok(_) = response.set_body_file(file) else {
        // We cannot process the index file
        return Response::new_500_internalservererror();
    };

    // Set headers and finalize request
    response.set_content_type("application/vnd.apple.mpegurl");
    response
}

/// Serves a GET request for a HLS entry
pub fn get_fragment(request: &Request, config: &Config) -> Response {
    // Extract fragment counter
    // Note: Fragments follow the format `/live-%08d.ts`, this allows for some optimization
    let Ok(target) = <[u8; 17]>::try_from(request.target.as_ref()) else {
        // The request target is not a valid, absolute fragment name
        return Response::new_404_notfound();
    };

    // Split path into segments
    let prefix = &target[0..6];
    let number = &target[6..14];
    let suffix = &target[14..17];
    let filename = &target[1..17];

    // Validate fragment name format
    let b"/live-" = prefix else {
        // The request target prefix is invalid
        return Response::new_404_notfound();
    };
    let true = number.iter().all(u8::is_ascii_digit) else {
        // The request target fragment counter is invalid
        return Response::new_404_notfound();
    };
    let b".ts" = suffix else {
        // The request target suffix is invalid
        return Response::new_404_notfound();
    };

    // Assemble path
    // Note: This can never fail as we have validated that the file name is valid
    let filename = str::from_utf8(&filename).expect("failed to parse ASCII filename");
    let path = config.RTSP2HLS_TEMPDIR.join(filename);

    // Open the file
    let Ok(file) = File::open(path) else {
        // We cannot open the fragment file
        return Response::new_404_notfound();
    };

    // Assemble the response
    let mut response = Response::new_200_ok();
    let Ok(_) = response.set_body_file(file) else {
        // We cannot process the index file
        return Response::new_500_internalservererror();
    };

    // Set headers and finalize request
    response.set_content_type("video/mp2t");
    response
}
