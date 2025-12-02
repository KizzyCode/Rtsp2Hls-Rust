//! The server config

use crate::error;
use crate::error::Error;
use std::borrow::Cow;
use std::env::{self, VarError};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

/// The server config
#[derive(Debug, Clone)]
#[allow(non_snake_case, reason = "We want to map the exact naming of the environment variables")]
pub struct Config {
    /// The RTSP server to stream from
    ///
    /// # Example
    /// An RTSP URL, e.g. `rtsps://192.168.178.69:322/streaming/live/1`.
    pub RTSP2HLS_SOURCE: Cow<'static, str>,
    /// The socket address to listen on for HLS HTTP requests
    ///
    /// # Example
    /// An `address:port` combination; defaults to [`Self::RTSP2HLS_LISTEN_DEFAULT`].
    pub RTSP2HLS_LISTEN: SocketAddr,
    /// The maximum amount of simultanous connections
    ///
    /// # Example
    /// The amount of connections, e.g. `64`; defaults to [`Self::RTSP2HLS_MAXCONN_DEFAULT`].
    pub RTSP2HLS_MAXCONN: usize,
    /// The canonicalized temp directory for HLS stream creation
    ///
    /// # Example
    /// The temp directory path, e.g. `/tmp/rtsp2hls`; defaults to [`Self::RTSP2HLS_TEMPDIR_DEFAULT`]. It is recommended
    /// to put the tempdir into an in-memory filesystem.
    pub RTSP2HLS_TEMPDIR: PathBuf,
}
impl Config {
    /// The default address if [`Self::RTSP2HLS_LISTEN`] is not specified
    pub const RTSP2HLS_LISTEN_DEFAULT: &str = "[::]:8080";
    /// The default amount of connections if [`Self::RTSP2HLS_MAXCONN`] is not specified
    pub const RTSP2HLS_MAXCONN_DEFAULT: &str = "1024";
    /// The default temp directpry path if [`Self::RTSP2HLS_TEMPDIR`] is not specified
    pub const RTSP2HLS_TEMPDIR_DEFAULT: &str = "/tmp/rtsp2hls";

    /// Gets the config from the environment
    pub fn from_env() -> Result<Self, Error> {
        Ok(Config {
            RTSP2HLS_SOURCE: Self::rtsp2hls_source()?,
            RTSP2HLS_LISTEN: Self::rtsp2hls_listen()?,
            RTSP2HLS_MAXCONN: Self::rtsp2hls_maxconn()?,
            RTSP2HLS_TEMPDIR: Self::rtsp2hls_tempdir()?,
        })
    }

    /// Parses the `RTSP2HLS_SOURCE` environment variable
    fn rtsp2hls_source() -> Result<Cow<'static, str>, Error> {
        Self::env("RTSP2HLS_SOURCE", None)
    }

    /// Parses the `RTSP2HLS_LISTEN` environment variable, or falls back to [`Self::RTSP2HLS_LISTEN_DEFAULT`]
    fn rtsp2hls_listen() -> Result<SocketAddr, Error> {
        let address = Self::env("RTSP2HLS_LISTEN", Some(Self::RTSP2HLS_LISTEN_DEFAULT))?;
        Ok(address.parse()?)
    }

    /// Parses the `RTSP2HLS_MAXCONN` environment variable, or falls back to [`Self::RTSP2HLS_MAXCONN_DEFAULT`]
    fn rtsp2hls_maxconn() -> Result<usize, Error> {
        let address = Self::env("RTSP2HLS_MAXCONN", Some(Self::RTSP2HLS_MAXCONN_DEFAULT))?;
        Ok(address.parse()?)
    }

    /// Parses the `RTSP2HLS_TEMPDIR` environment variable, or falls back to [`Self::RTSP2HLS_TEMPDIR_DEFAULT`]
    fn rtsp2hls_tempdir() -> Result<PathBuf, Error> {
        let tempdir = Self::env("RTSP2HLS_TEMPDIR", Some(Self::RTSP2HLS_TEMPDIR_DEFAULT))?;
        let tempdir_canonicalized = Path::new(tempdir.as_ref()).canonicalize()?;
        Ok(tempdir_canonicalized)
    }

    /// Gets the environment variable with the given name or returns the default value
    fn env(name: &str, default: Option<&'static str>) -> Result<Cow<'static, str>, Error> {
        match (env::var(name), default) {
            (Ok(value), _) => Ok(Cow::Owned(value)),
            (Err(VarError::NotPresent), Some(default)) => Ok(Cow::Borrowed(default)),
            (Err(VarError::NotPresent), _) => Err(error!(r#"Missing environment variable "{name}""#)),
            (Err(e), _) => Err(error!(with: e, r#"Invalid environment variable "{name}""#)),
        }
    }
}
