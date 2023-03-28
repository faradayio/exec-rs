//! A simple wrapper around the C library's `execvp` function.
//!
//! For examples, see [the repository](https://github.com/faradayio/exec-rs).
//!
//! We'd love to fully integrate this with `std::process::Command`, but
//! that module doesn't export sufficient hooks to allow us to add a new
//! way to execute a program.

extern crate errno;
extern crate libc;

use errno::{errno, Errno};
use std::error;
use std::ffi::{CString, NulError, OsStr, OsString};
use std::fmt;
use std::iter::{IntoIterator, Iterator};
use std::os::unix::ffi::OsStrExt;
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
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            &Error::BadArgument(ref err) => Some(err),
            &Error::Errno(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::BadArgument(ref err) => write!(f, "{}: {}", self.to_string(), err),
            &Error::Errno(err) => write!(f, "{}: {}", self.to_string(), err),
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
pub fn execvp<S, I>(program: S, args: I) -> Error
where
    S: AsRef<OsStr>,
    I: IntoIterator,
    I::Item: AsRef<OsStr>,
{
    // Add null terminations to our strings and our argument array,
    // converting them into a C-compatible format.
    let program_cstring = exec_try!(to_program_cstring(program));
    let argv = exec_try!(to_argv(args));

    // Use an `unsafe` block so that we can call directly into C.
    let res = unsafe { libc::execvp(program_cstring.as_ptr(), argv.char_ptrs.as_ptr()) };

    // Handle our error result.
    if res < 0 {
        Error::Errno(errno())
    } else {
        // Should never happen.
        panic!("execvp returned unexpectedly")
    }
}

/// Run `program` with `args` and environment `envs`, completely replacing
/// the currently running program. If it returns at all, it always return
/// an error.
///
/// Note that `program` and the first element of `args` will normally be
/// identical. The former is the program we ask the operating system to
/// run, and the latter is the value that will show up in `argv[0]` when
/// the program executes. On POSIX systems, these can technically be
/// completely different, and we've preserved that much of the low-level
/// API here.
///
/// # Examples
///
/// ```no_run
/// use std::env::vars_os;
/// use std::ffi::OsString;
/// let err = execvpe(
///     "bash",
///     ["bash"],
///     vars_os().chain([(OsString::from("NAME"), OsString::from("VALUE"))]),
/// println!("Error: {}", err);
/// ```
#[cfg(not(target_os = "macos"))]
pub fn execvpe<S, I, J, N, V>(program: S, args: I, envs: J) -> Error
where
    S: AsRef<OsStr>,
    I: IntoIterator,
    I::Item: AsRef<OsStr>,
    J: IntoIterator<Item = (N, V)>,
    N: AsRef<OsStr> + std::fmt::Debug,
    V: AsRef<OsStr> + std::fmt::Debug,
{
    // Add null terminations to our strings and our argument array,
    // converting them into a C-compatible format.
    let program_cstring = exec_try!(to_program_cstring(program));
    let argv = exec_try!(to_argv(args));
    let envp = exec_try!(to_envp(envs));

    // Use an `unsafe` block so that we can call directly into C.
    let res = unsafe {
        libc::execvpe(
            program_cstring.as_ptr(),
            argv.char_ptrs.as_ptr(),
            envp.char_ptrs.as_ptr(),
        )
    };

    // Handle our error result.
    if res < 0 {
        Error::Errno(errno())
    } else {
        // Should never happen.
        panic!("execvp returned unexpectedly")
    }
}

fn to_program_cstring<S>(program: S) -> std::result::Result<CString, NulError>
where
    S: AsRef<OsStr>,
{
    CString::new(program.as_ref().as_bytes())
}

// Struct ensures that cstrings have same lifetime as char_ptrs that points into them
struct Argv {
    #[allow(dead_code)]
    cstrings: Vec<CString>,
    char_ptrs: Vec<*const i8>,
}

fn to_argv<I>(args: I) -> std::result::Result<Argv, NulError>
where
    I: IntoIterator,
    I::Item: AsRef<OsStr>,
{
    let cstrings = args
        .into_iter()
        .map(|arg| CString::new(arg.as_ref().as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;

    let mut char_ptrs = cstrings.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
    char_ptrs.push(ptr::null());

    Ok(Argv {
        cstrings: cstrings,
        char_ptrs: char_ptrs,
    })
}

// Struct ensures that cstrings have same lifetime as char_ptrs that points into them
#[cfg(not(target_os = "macos"))]
struct Envp {
    #[allow(dead_code)]
    cstrings: Vec<CString>,
    char_ptrs: Vec<*const i8>,
}

#[cfg(not(target_os = "macos"))]
fn to_envp<J, N, V>(envs: J) -> std::result::Result<Envp, NulError>
where
    J: IntoIterator<Item = (N, V)>,
    N: AsRef<OsStr> + std::fmt::Debug,
    V: AsRef<OsStr> + std::fmt::Debug,
{
    let cstrings = envs
        .into_iter()
        .map(|(n, v)| {
            let mut temp: OsString = OsString::new();
            temp.push(n);
            temp.push("=");
            temp.push(v);
            CString::new(temp.as_bytes())
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut char_ptrs = cstrings.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();
    char_ptrs.push(ptr::null());

    Ok(Envp {
        cstrings: cstrings,
        char_ptrs: char_ptrs,
    })
}

/// Build a command to execute.  This has an API which is deliberately
/// similar to `std::process::Command`.
///
/// ```no_run
/// let err = exec::Command::new("echo")
///     .arg("hello")
///     .arg("world")
///     .exec();
/// println!("Error: {}", err);
/// ```
///
/// If the `exec` function succeeds, it will never return.
pub struct Command {
    /// The program name and arguments, in typical C `argv` style.
    argv: Vec<OsString>,
}

impl Command {
    /// Create a new command builder, specifying the program to run.  The
    /// program will be searched for using the usual rules for `PATH`.
    pub fn new<S: AsRef<OsStr>>(program: S) -> Command {
        Command {
            argv: vec![program.as_ref().to_owned()],
        }
    }

    /// Add an argument to the command builder.  This can be chained.
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Command {
        self.argv.push(arg.as_ref().to_owned());
        self
    }

    /// Add multiple arguments to the command builder.  This can be
    /// chained.
    ///
    /// ```no_run
    /// let err = exec::Command::new("echo")
    ///     .args(&["hello", "world"])
    ///     .exec();
    /// println!("Error: {}", err);
    /// ```
    pub fn args<S: AsRef<OsStr>>(&mut self, args: &[S]) -> &mut Command {
        for arg in args {
            self.arg(arg.as_ref());
        }
        self
    }

    /// Execute the command we built.  If this function succeeds, it will
    /// never return.
    pub fn exec(&mut self) -> Error {
        execvp(&self.argv[0], &self.argv)
    }
}
