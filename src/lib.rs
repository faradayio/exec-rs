//! A simple wrapper around the C library's `execvp` function.
//!
//! For examples, see [the repository](https://github.com/faradayio/exec-rs).
//!
//! We'd love to fully integrate this with `std::process::Command`, but
//! that module doesn't export sufficient hooks to allow us to add a new
//! way to execute a program.

extern crate errno;
extern crate libc;

use errno::{Errno, errno};
use std::error;
use std::error::Error as ErrorTrait; // Include for methods, not name.
use std::ffi::{CString, NulError};
use std::iter::{IntoIterator, Iterator};
use std::fmt;
use std::ptr;

/// Represents an error calling `exec`.
///
/// This is marked `#[must_use]`, which is unusual for error types.
/// Normally, the fact that `Result` is marked in this fashion is
/// sufficient, but in this case, this error is returned bare from
/// functions that only return a result if they fail.
#[derive(Debug)]
#[must_use]
pub enum Error {
    /// One of the strings passed to `execv` contained an internal null byte
    /// and can't be passed correctly to C.
    BadArgument(NulError),
    /// An error was returned by the system.
    Errno(Errno),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::BadArgument(_) => "bad argument to exec",
            &Error::Errno(_) => "couldn't exec process",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::BadArgument(ref err) => Some(err),
            &Error::Errno(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::BadArgument(ref err) =>
                write!(f, "{}: {}", self.description(), err),
            &Error::Errno(err) =>
                write!(f, "{}: {}", self.description(), err),
        }
    }
}

impl From<NulError> for Error {
    /// Convert a `NulError` into an `ExecError`.
    fn from(err: NulError) -> Error {
        Error::BadArgument(err)
    }
}

/// Like `try!`, but it just returns the error directly without wrapping it
/// in `Err`.  For functions that only return if something goes wrong.
macro_rules! exec_try {
    ( $ expr : expr ) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return From::from(err),
        }
    };
}

/// Run `program` with `args`, completely replacing the currently running
/// program.  If it returns at all, it always returns an error.
///
/// Note that `program` and the first element of `args` will normally be
/// identical.  The former is the program we ask the operating system to
/// run, and the latter is the value that will show up in `argv[0]` when
/// the program executes.  On POSIX systems, these can technically be
/// completely different, and we've perserved that much of the low-level
/// API here.
///
/// # Examples
///
/// ```no_run
/// let err = exec::execvp("echo", &["echo", "foo"]);
/// println!("Error: {}", err);
/// ```
///
/// If we wanted to take our command-line arguments and treat them as a
/// program to run, we could do it like this:
///
/// ```no_run
/// use std::env;
///
/// let args: Vec<String> = env::args().skip(1).collect();
/// let program = args[0].clone();
/// let err = exec::execvp(program, &args);
/// println!("Error: {}", err);
/// ```
pub fn execvp<S, I>(program: S, args: I) -> Error
    where S: AsRef<str>, I: IntoIterator, I::Item: AsRef<str>
{
    // Add null terminations to our strings and our argument array,
    // converting them into a C-compatible format.
    let program_cstring = exec_try!(CString::new(program.as_ref()));
    let arg_cstrings = exec_try!(args.into_iter().map(|arg| {
        CString::new(arg.as_ref())
    }).collect::<Result<Vec<_>, _>>());
    let mut arg_charptrs: Vec<_> = arg_cstrings.iter().map(|arg| {
        arg.as_bytes_with_nul().as_ptr() as *const i8
    }).collect();
    arg_charptrs.push(ptr::null());

    // Use an `unsafe` block so that we can call directly into C.
    let res = unsafe {
        libc::execvp(program_cstring.as_bytes_with_nul().as_ptr() as *const i8,
                     arg_charptrs.as_ptr())
    };

    // Handle our error result.
    if res < 0 {
        Error::Errno(errno())
    } else {
        // Should never happen.
        panic!("execvp returned unexpectedly")
    }
}
