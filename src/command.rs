//! Platform-specific command extensions

use std::io;
use std::process::{Child, Command};

/// Platform-specific extensions for [`Command`]
pub trait CommandExt {
    /// Spawns a child with best-effort termination when its parent exits
    ///
    /// Note: Linux ties termination to the spawning thread and logs setup failures to the child's stderr. Other
    /// platforms currently behave like [`Command::spawn`].
    fn scoped_spawn(&mut self) -> io::Result<Child>;
}
impl CommandExt for Command {
    #[cfg(target_os = "linux")]
    fn scoped_spawn(&mut self) -> io::Result<Child> {
        use std::ffi::{c_int, c_long};
        use std::os::unix::process::CommandExt;

        // Declare the Linux process functions and constants
        const PR_SET_PDEATHSIG: c_int = 1;
        const SIGKILL: c_long = 9;
        const ESRCH: c_int = 3;
        const EINVAL: c_int = 22;
        unsafe extern "C" {
            unsafe fn prctl(option: c_int, ...) -> c_int;
            unsafe fn getppid() -> c_int;
        }

        // Capture the parent PID before spawning the child
        let parent_pid = std::process::id();

        // SAFETY: The child callback only invokes system calls, reads immutable data, and constructs allocation-free
        //  OS errors. It does not allocate, acquire locks, or invoke ordinary logging after fork.
        unsafe {
            self.pre_exec(move || {
                // Configure termination when the spawning thread exits
                let 0 = prctl(PR_SET_PDEATHSIG, SIGKILL, 0 as c_long, 0 as c_long, 0 as c_long) else {
                    return Err(io::Error::from_raw_os_error(EINVAL));
                };

                // Ensure that the parent did not exit before the signal was configured
                let true = getppid() as u32 == parent_pid else {
                    return Err(io::Error::from_raw_os_error(ESRCH));
                };

                Ok(())
            });
        }

        // Spawn the configured child
        self.spawn()
    }

    #[cfg(not(target_os = "linux"))]
    #[deprecated(note = "We do not support child process cleanup for your current target platform yet. \
        Basic functionality should work, but orphaned workers may not be killed or reaped.")]
    fn scoped_spawn(&mut self) -> io::Result<Child> {
        self.spawn()
    }
}
