//! Implements the crate's error type

use std::backtrace::{Backtrace, BacktraceStatus};
use std::error;
use std::fmt::{self, Display, Formatter};

/// Creates a new error
#[macro_export]
macro_rules! error {
    (with: $error:expr, $($arg:tt)*) => {{
        let error = format!($($arg)*);
        let source = Box::new($error);
        $crate::error::Error::new(error, Some(source))
    }};
    ($($arg:tt)*) => {{
        let error = format!($($arg)*);
        $crate::error::Error::new(error, None)
    }};
}

/// The crates error type
#[derive(Debug)]
pub struct Error {
    /// The error description
    pub error: String,
    /// The underlying error
    pub source: Option<Box<dyn error::Error + Send>>,
    /// The backtrace
    pub backtrace: Backtrace,
}
impl Error {
    /// Creates a new error
    #[doc(hidden)]
    pub fn new(error: String, source: Option<Box<dyn error::Error + Send>>) -> Self {
        let backtrace = Backtrace::capture();
        Self { error, source, backtrace }
    }

    /// Logs `self` to stderr
    pub fn log_to_stderr(&self) {
        eprintln!("{self}");
        if self.backtrace.status() == BacktraceStatus::Captured {
            // Print the backtrace if any
            eprint!("{}", self.backtrace);
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Print the error
        writeln!(f, "{}", self.error)?;

        // Print the source
        if let Some(source) = &self.source {
            writeln!(f, " caused by: {source}")?;
        }
        Ok(())
    }
}
impl<T> From<T> for Error
where
    T: error::Error + Send + 'static,
{
    fn from(source: T) -> Self {
        let error = source.to_string();
        let source = Box::new(source);
        Self::new(error, Some(source))
    }
}
