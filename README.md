[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/Rtsp2Hls-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/Rtsp2Hls-rust)
<!--
[![docs.rs](https://docs.rs/rtsp2hls/badge.svg)](https://docs.rs/rtsp2hls)
[![crates.io](https://img.shields.io/crates/v/rtsp2hls.svg)](https://crates.io/crates/rtsp2hls)
[![Download numbers](https://img.shields.io/crates/d/rtsp2hls.svg)](https://crates.io/crates/rtsp2hls)
[![dependency status](https://deps.rs/crate/rtsp2hls/latest/status.svg)](https://deps.rs/crate/rtsp2hls)
-->

# `rtsp2hls`
Welcome to `rtsp2hls` 🎉

`rtsp2hls` is a trivial wrapper-application around `ffmpeg` to fetch an `rtsp://`-stream and serve it as HLS livestream
**without** reencoding. It works by invoking `ffmpeg` to create a filesystem-backed HLS stream, and then provides a
simple HTTP server to serve the playlist. The application can be used to e.g. transform video streams from IP cameras or
similar into an HLS livestream you can simply view or embed in a browser.

## Usage
The application is configured via environment variables only:
- `RTSP2HLS_SOURCE`: The RTSP source URL, e.g. `rtsps://192.168.178.69:322/streaming/live/1`. This parameter is
  **required**.
- `RTSP2HLS_LISTEN`: The address for the HTTP/HLS server to listen on. This parameter is optional, and defaults to
  `[::]:8080`.
- `RTSP2HLS_MAXCONN`: The maximum amount of simultaneous connections the HTTP/HLS server will accept. This parameter is
  optional, and defaults to `1024`.
- `RTSP2HLS_TEMPDIR`: The temp directory for `ffmpeg` to write the HLS playlist to. This parameter is optional, and
  defaults to `/tmp/rtsp2hls`. Note: As the folder contains only temporary data, but has continious I/O, it is
  recommended to put it onto a memory-backed filesystem.

## Security Considerations
- **No TLS verification**: Due to limitations in `ffmpeg`, it does not validate TLS certificates for
  `rtsps://`-connections. This allows for MitM-attacks between `rtsp2hls` and your stream source. If that is a security
  concern, it is recommended to use a secure overlay network to protect this link.
- **No authentication for the HTTP/HLS server**: The HTTP/HLS server does not provide an authentication layer for
  incoming requests. If that is a security concern, it is recommended to put the server behind an authentication proxy.
