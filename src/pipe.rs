use crate::fork::{Fork, ForkPid};

const FILE_NULL: *mut libc::FILE = std::ptr::null_mut::<libc::FILE>();

#[derive(Debug, Default, Clone)]
pub struct Pipe {
    write_end: libc::pid_t,
    read_end: libc::pid_t,
}

impl Pipe {
    fn pipe() -> Option<Pipe> {
        let mut end = [0; 2];
        match unsafe { libc::pipe(end.as_mut_ptr()) } {
            libc::INT_MIN..=-1 => None,
            _ => Some(Pipe {
                write_end: end[1],
                read_end: end[0],
            }),
        }
    }

    fn pipe2(flag: i32) -> Option<Pipe> {
        let mut end = [0; 2];
        match unsafe { libc::pipe2(end.as_mut_ptr(), flag) } {
            std::i32::MIN..=-1 => None,
            _ => Some(Pipe {
                write_end: end[1],
                read_end: end[0],
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Popen {
    arg: String,
    pid: Option<libc::pid_t>,
    stdin: *mut libc::FILE,
    stdout: *mut libc::FILE,
    stderr: *mut libc::FILE,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PopenError {
    PipeCreateFailed,
    ExecArgFailed,
    ForkFailed,
}

impl PopenError {
    fn as_str(&self) -> &'static str {
        match *self {
            Self::PipeCreateFailed => "create pipe failed!",
            Self::ExecArgFailed => "exec arg failed!",
            Self::ForkFailed => "fork failed!",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct RedirectPipe {
    stdout: Pipe,
    stdin: Pipe,
    stderr: Pipe,
}

impl RedirectPipe {
    pub fn new() -> Option<RedirectPipe> {
        Some(RedirectPipe {
            stdout: Pipe::pipe()?,
            stdin: Pipe::pipe()?,
            stderr: Pipe::pipe()?,
        })
    }
}

impl Popen {
    pub fn new(arg: &str) -> Result<Popen, PopenError> {
        let RedirectPipe {
            stdout,
            stdin,
            stderr,
        } = RedirectPipe::new().ok_or(PopenError::PipeCreateFailed)?;
        let mut popen = Popen {
            arg: String::from(arg),
            pid: None,
            stdin: FILE_NULL,
            stdout: FILE_NULL,
            stderr: FILE_NULL,
        };
        match Fork::fork() {
            Some(ForkPid::Parent(_)) => Ok(popen),
            Some(ForkPid::Children(pid)) => {
                popen.pid = Some(pid);
                Err(PopenError::ExecArgFailed)
            }
            None => Err(PopenError::ForkFailed),
        }
    }
}

impl Drop for Popen {
    fn drop(&mut self) {
        self.pid = None;
        unsafe {
            libc::fclose(self.stdin);
            libc::fclose(self.stdout);
            libc::fclose(self.stderr);
        }
    }
}

#[cfg(tests)]
mod tests {}
