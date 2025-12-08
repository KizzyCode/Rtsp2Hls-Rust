//! RTSP client task

use crate::config::Config;
use crate::error;
use crate::error::Error;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{self, Child, Command};
use std::time::Duration;
use std::{fs, mem, thread};

/// An RTSP client to create a filesystem-backed HLS stream from an RTSP source
#[derive(Debug)]
pub struct RtspClient {
    /// The temp directory
    tempdir: PathBuf,
    /// The client worker process
    worker: RtspClientProcess,
}
impl RtspClient {
    /// The watchdog period (currently we give a grace interval of 10 fragments)
    pub const WATCHDOG_PERIOD: Duration = Duration::from_secs(RtspClientProcess::SEGMENT_LENGTH.as_secs() * 10);

    /// Creates a new RTSP client with the given RTSP URL
    pub fn new(config: &Config) -> Result<Self, Error> {
        let worker = RtspClientProcess::new(config)?;
        Ok(Self { tempdir: config.RTSP2HLS_TEMPDIR.clone(), worker })
    }

    /// Starts a continous watchdog over `self`
    pub fn start_watchdog(mut self) -> ! {
        let mut hls_snapshot = BTreeSet::new();
        loop {
            // Perform periodic healthcheck
            thread::sleep(RtspClient::WATCHDOG_PERIOD);
            let Ok(true) = self.worker.is_alive() else {
                error!("The RTSP client terminated unexpectedly").log_to_stderr();
                process::exit(2);
            };

            // Create a current HLS livestream snapshot
            let Ok(mut hls_snapshot_new) = self.find_ts_files() else {
                error!("Failed to perform RTSP client healthcheck").log_to_stderr();
                process::exit(2);
            };

            // Ensure that the HLS stream has been updated
            mem::swap(&mut hls_snapshot_new, &mut hls_snapshot);
            let false = hls_snapshot == hls_snapshot_new else {
                error!("The RTSP client has stalled").log_to_stderr();
                process::exit(2);
            };
        }
    }

    /// Returns a list of all `.ts`-files
    fn find_ts_files(&self) -> Result<BTreeSet<OsString>, Error> {
        let directory = fs::read_dir(&self.tempdir)?;
        let ts_files: BTreeSet<_> = (directory.flatten())
            .map(|directory_entry| directory_entry.file_name())
            .filter(|name| name.as_encoded_bytes().ends_with(b".ts"))
            .collect();
        Ok(ts_files)
    }
}

/// A `gstreamer` worker process for [`RtspClient`]
#[derive(Debug)]
struct RtspClientProcess {
    /// The child process
    child: Child,
}
impl RtspClientProcess {
    /// The desired length of each HLS segment
    const SEGMENT_LENGTH: Duration = Duration::from_secs(1);
    /// The amount of HLS-ts segments to retain
    const SEGMENT_COUNT: u32 = 2;

    /// Creates a new RTSP-to-HLS client for the given RTSP source URL
    pub fn new(config: &Config) -> Result<Self, Error> {
        // Assemble combined arguments
        let rtspsrc = format!("location={}", config.RTSP2HLS_SOURCE);
        let max_files = format!("max-files={}", Self::SEGMENT_COUNT);
        let playlist_length = format!("playlist-length={}", Self::SEGMENT_COUNT);
        let target_duration = format!("target-duration={}", Self::SEGMENT_LENGTH.as_secs());

        // Select TLS validation flags
        // See https://docs.gtk.org/gio/flags.TlsCertificateFlags.html
        let tls_validation_flags = match config.RTSP2HLS_VERIFYTLS {
            true => "tls-validation-flags=127", // full validation
            false => "tls-validation-flags=0",  // no validation
        };

        // Spawn worker
        let child = Command::new("gst-launch-1.0")
            // Create RTSP source with TLS validation configuration
            .arg("rtspsrc").arg(rtspsrc).arg(tls_validation_flags)
            // Decode RTSP stream with h.264 payload into bitstream
            .arg("!").arg("queue").arg("!").arg("rtph264depay")
            // Decode h.264 bistream and remux it to MPEG-TS segments
            .arg("!").arg("h264parse").arg("!").arg("mpegtsmux")
            // Create an HLS livestream sink from the MPEG-TS segment stream
            .arg("!").arg("hlssink").arg(max_files).arg(playlist_length).arg(target_duration)
            // Specify playlist and fragment paths relativ to the working dir
            .arg("playlist-location=index.m3u8").arg("location=live-%08d.ts")
            // Spawn within tempdir as our working dir
            .current_dir(&config.RTSP2HLS_TEMPDIR).spawn()?;

        // Init self
        Ok(Self { child })
    }

    /// Checks if the child process is still alive
    pub fn is_alive(&mut self) -> Result<bool, Error> {
        let status = self.child.try_wait()?;
        Ok(status.is_none())
    }
}
impl Drop for RtspClientProcess {
    fn drop(&mut self) {
        // Best-effort to kill child process
        let _ = self.child.kill();
    }
}
