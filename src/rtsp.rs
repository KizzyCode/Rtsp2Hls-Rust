//! RTSP client task

use crate::error;
use crate::error::Error;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command};
use std::time::Duration;
use std::{fs, mem, thread};

/// An RTSP client to create a filesystem-backed HLS stream from an RTSP source
#[derive(Debug)]
pub struct RtspClient {
    /// The temp directory
    tempdir: PathBuf,
    /// The current `ffmpeg` worker
    worker: RtspClientProcess,
}
impl RtspClient {
    /// The watchdog period (currently we give a grace interval of 3 fragments)
    pub const WATCHDOG_PERIOD: Duration = Duration::from_secs(RtspClientProcess::SEGMENT_LENGTH.as_secs() * 3);

    /// Creates a new RTSP client with the given RTSP URL
    pub fn new(rtsp: &str, tempdir: &Path) -> Result<Self, Error> {
        let worker = RtspClientProcess::new(rtsp, tempdir)?;
        Ok(Self { tempdir: tempdir.to_owned(), worker })
    }

    /// Starts a continous watchdog over `self`
    pub fn start_watchdog(mut self) -> ! {
        let mut hls_snapshot = BTreeSet::new();
        loop {
            // Perform periodic healthcheck
            thread::sleep(RtspClient::WATCHDOG_PERIOD);
            let Ok(true) = self.worker.is_alive() else {
                error!("The `ffmpeg` worker terminated unexpectedly").log_to_stderr();
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
                error!("The `ffmpeg` worker has stalled").log_to_stderr();
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

/// An `ffmpeg` worker process for [`RtspClient`]
#[derive(Debug)]
struct RtspClientProcess {
    /// The `ffmpeg` child
    child: Child,
}
impl RtspClientProcess {
    /// The desired length of each HLS segment
    const SEGMENT_LENGTH: Duration = Duration::from_secs(1);
    /// The amount of HLS-ts segments to retain
    const SEGMENT_COUNT: u32 = 2;
    /// The `ffmpeg` log level to use
    const LOG_LEVEL: &str = "warning";

    /// Creates a new RTSP-to-HLS client for the given RTSP source URL
    pub fn new(rtsp: &str, tempdir: &Path) -> Result<Self, Error> {
        // Spawn ffmpeg worker
        let child = Command::new("ffmpeg")
            // Set RTSP source URL
            .arg("-i").arg(rtsp)
            // Don't reencode segments, keep them as-is
            .arg("-c:v").arg("copy").arg("-c:a").arg("copy")
            // Specify HLS fragment size
            .arg("-hls_time").arg(Self::SEGMENT_LENGTH.as_secs().to_string())
            // Ensure that the HLS fragments have deterministic names
            .arg("-hls_segment_filename").arg("live-%08d.ts")
            // Only keep a certain amount of segments, delete older ones
            .arg("-hls_list_size").arg(Self::SEGMENT_COUNT.to_string())
            .arg("-hls_flags").arg("delete_segments")
            // Only display error messages
            .arg("-loglevel").arg(Self::LOG_LEVEL)
            // Write the index to `index.m3u8`
            .arg("index.m3u8")
            // Spawn inside tempdir
            .current_dir(tempdir).spawn()?;

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
