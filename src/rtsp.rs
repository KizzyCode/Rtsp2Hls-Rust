//! RTSP client task

use crate::error;
use crate::error::Error;
use std::path::Path;
use std::process::{self, Child, Command};
use std::thread;
use std::time::Duration;

/// An `ffmpeg` client to create a filesystem-backed HLS stream from an RTSP source
#[derive(Debug)]
pub struct RtspClient {
    /// The `ffmpeg` command to execute
    command: Command,
}
impl RtspClient {
    /// The desired length of each HLS segment
    const SEGMENT_LENGTH: Duration = Duration::from_secs(1);
    /// The amount of HLS-ts segments to retain
    const SEGMENT_COUNT: usize = 2;
    /// The `ffmpeg` log level to use
    const LOG_LEVEL: &str = "warning";

    /// Creates a new RTSP-to-HLS client for the given RTSP source URL
    pub fn new(rtsp: &str, tempdir: &Path) -> Self {
        // Build `ffmpeg` command line
        let mut command = Command::new("ffmpeg");
        command.current_dir(tempdir)
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
            .arg("index.m3u8");

        // Init self
        Self { command }
    }

    /// Starts the `ffmpeg` task
    pub fn spawn(mut self) -> Result<RtspClientTask, Error> {
        let task = self.command.spawn()?;
        Ok(RtspClientTask { task })
    }
}

/// A running [`RtspClient`] task
#[derive(Debug)]
pub struct RtspClientTask {
    /// The child process
    task: Child,
}
impl RtspClientTask {
    /// Detaches the task into background
    pub fn detach(mut self) {
        thread::spawn(move || {
            // Wait for the process to terminate (which should not happen unless an error occurs)
            let error = match self.task.wait() {
                Ok(status) => error!("ffmpeg task stopped unexpectedly: {status:?}"),
                Err(e) => error!(with: e, "failed to poll ffmpeg task"),
            };

            // Log the error and terminate
            error.log_to_stderr();
            process::exit(2);
        });
    }
}
