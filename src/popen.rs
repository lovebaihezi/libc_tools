use libc::{
    __errno_location, _exit, c_int, c_void, execl, fclose, fdopen, pipe, pipe2, read, socketpair,
    AF_UNIX, FILE, O_NONBLOCK, SOCK_STREAM, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};
use std::ffi::{CString, NulError};

use crate::{
    create_pipe, create_pipe2, dup::DupError, wait::Wait, Close, Dup, Fork, ForkPid,
    SocketPairError,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Popen {
    pub arg: String,
    pub stdin: *mut FILE,
    pub stdout: *mut FILE,
    pub stderr: *mut FILE,
    pub pid: Option<c_int>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PopenError {
    PipeCreateFailed,
    ExecArgFailed(c_int),
    ForkFailed,
    PipeRedirectFailed(c_int),
    Dup2Errno(DupError),
    FdOpenErrno(c_int),
    CloseError(Close),
    SocketPairError(SocketPairError),
    CreateRedirectError(c_int),
    CStringParesError(NulError),
}

impl PopenError {
    fn to_string(&self) -> String {
        match self {
            Self::PipeCreateFailed => "create pipe failed!".to_string(),
            Self::ExecArgFailed(code) => std::format!("exec arg failed! exit code: {}", code),
            Self::ForkFailed => "fork failed!".to_string(),
            Self::PipeRedirectFailed(v) => {
                std::format!("redirect std(in|out|err) to pipe failed! code: {}", v)
            }
            PopenError::Dup2Errno(v) => std::format!("dup2 old fd to new fd failed! {}", v),
            Self::FdOpenErrno(v) => {
                std::format!("open file descriptor as file* stream failed! errno: {}", v)
            }
            Self::CloseError(v) => {
                std::format!("{}", v)
            }
            Self::SocketPairError(v) => {
                std::format!("socket pair {}", v)
            }
            Self::CreateRedirectError(v) => {
                std::format!(
                    "create socket pair and stderr pipe both failed! errno: {}",
                    v
                )
            }
            Self::CStringParesError(n) => {
                std::format!("parse {:<.20} failed!", n.to_string())
            }
        }
    }
}

// unsafe fn create_pipe() -> Result<[[libc::c_int; 2]; 3], PopenError> {
//     let mut pipes = [[-1; 2]; 3];
//     match (
//         pipe(pipes[0].as_mut_ptr()),
//         pipe(pipes[1].as_mut_ptr()),
//         pipe(pipes[2].as_mut_ptr()),
//     ) {
//         (-1, _, _) | (_, -1, _) | (_, _, -1) => Err(PopenError::PipeCreateFailed),
//         _ => Ok(pipes),
//     }
// }

fn socket_pipe() -> Result<[[c_int; 2]; 2], PopenError> {
    let mut sv = [0 as c_int; 2];
    let mut fd = [0 as c_int; 2];
    match unsafe {
        (
            socketpair(AF_UNIX, SOCK_STREAM, 0, sv.as_mut_ptr()),
            pipe2(fd.as_mut_ptr(), O_NONBLOCK),
        )
    } {
        (-1, 0) => Err(PopenError::SocketPairError(SocketPairError::SocketErrno(
            unsafe { *__errno_location() },
        ))),
        (0, -1) => Err(PopenError::PipeCreateFailed),
        (-1, -1) => Err(PopenError::CreateRedirectError(unsafe {
            *__errno_location()
        })),
        (0, 0) => Ok([sv, fd]),
        _ => panic!("this should reached!"),
    }
}

// #[deprecated(note = "do not use!")]
impl Popen {
    pub fn arg(arg: &str) -> Box<Popen> {
        Box::new(Popen {
            arg: String::from(arg),
            stdin: 0 as *mut FILE,
            stdout: 0 as *mut FILE,
            stderr: 0 as *mut FILE,
            pid: None,
        })
    }
    pub fn exec(mut self: Box<Popen>) -> Result<Box<Popen>, PopenError> {
        // let [sv, fd] = socket_pipe()?;
        let [stdout, stdin] = create_pipe!(2).ok_or(PopenError::PipeCreateFailed)?;
        let [stderr] = create_pipe2!(1, [O_NONBLOCK]).ok_or(PopenError::PipeCreateFailed)?;
        match Fork::fork() {
            ForkPid::Parent((_, children)) => {
                self.pid = Some(children);
                Close::close(&[stdin[0], stdout[1], stderr[1]])
                    .or_else(|x| Err(PopenError::CloseError(x)))?;
                let r = CString::new("r").or_else(|x| Err(PopenError::CStringParesError(x)))?;
                let w = CString::new("w").or_else(|x| Err(PopenError::CStringParesError(x)))?;
                self.stdin = unsafe { fdopen(stdin[1], w.as_ptr()) };
                self.stdout = unsafe { fdopen(stdout[0], r.as_ptr()) };
                self.stderr = unsafe { fdopen(stderr[0], r.as_ptr()) };
                Ok(self)
            }
            // socket provide
            ForkPid::Children(_) => {
                Dup::dup2s(
                    &[stdout[1], stderr[1], STDIN_FILENO],
                    &[STDOUT_FILENO, STDERR_FILENO, stdin[0]],
                )
                .unwrap();
                Close::close(&[
                    stdout[0], stdout[1], stdin[0], stdin[1], stderr[0], stderr[1],
                ])
                .unwrap();
                let path = CString::new("/bin/sh").unwrap();
                let sh = CString::new("sh").unwrap();
                let exec = CString::new("-c").unwrap();
                let arg = CString::new(self.arg.clone()).unwrap();
                unsafe {
                    _exit(execl(
                        path.as_ptr(),
                        sh.as_ptr(),
                        exec.as_ptr(),
                        arg.as_ptr(),
                        0,
                    ))
                };
            }
            ForkPid::None => Err(PopenError::ForkFailed),
        }
    }
}

impl Drop for Popen {
    fn drop(&mut self) {
        let null = std::ptr::null_mut::<FILE>();
        for i in &[self.stdin, self.stdout, self.stderr][..] {
            if *i != null {
                unsafe { fclose(*i) };
            }
        }
        if let Some(pid) = self.pid {
            // eprintln!("pid: {}", pid);
            while {
                match Wait::children_with(pid, 0) {
                    Err(Wait::WaitFailure(libc::EINTR)) => true,
                    _ => false,
                }
            } {}
        } else {
            eprintln!("pid is missed!");
        }
    }
}

#[cfg(test)]
mod popen {
    use libc::{
        __errno_location, fclose, fgets, perror, socketpair, strlen, FILE, PF_UNIX, SOCK_CLOEXEC,
        SOCK_DGRAM,
    };

    use crate::{popen::popen, Close, Popen, Wait};

    #[test]
    // #[ignore = "absolutely correct"]
    fn test_libc_popen() {
        unsafe {
            // the common libc only support the 'r' and 'w', but the apple Libc support '+' (with socket)
            let stream = libc::popen(
                "echo hello\0".as_ptr() as *mut i8,
                "r\0".as_ptr() as *mut i8,
            );
            assert!(stream != std::ptr::null_mut::<libc::FILE>());
            let mut buf: [libc::c_char; 4096] = [0; 4096];
            while {
                let ptr = libc::fgets(buf.as_mut_ptr(), 4096, stream);
                ptr != std::ptr::null_mut::<libc::c_char>() && *ptr != libc::EOF as i8
            } {
                let len = libc::strlen(buf.as_ptr());
                assert!(len != 0);
                let str = "hello\0";
                for i in 0..len - 1 {
                    let x = &str[i..i + 1];
                    assert!(x.as_bytes()[0] == buf[i] as u8);
                }
                assert!(libc::strcmp(buf.as_ptr(), "hello\0".as_ptr() as *const i8) > 0);
            }
            assert!(libc::fclose(stream) != -1);
        }
    }
    #[test]
    fn test_popen_date() {
        Popen::arg("date").exec().unwrap();
    }

    #[test]
    fn socketpair_redirect() {
        unsafe {
            let mut fd: [i32; 4096] = [0; 4096];
            socketpair(PF_UNIX, SOCK_DGRAM, 0, fd.as_mut_ptr());
            Close::close(&[fd[0]]).unwrap();
        }
    }

    #[test]
    fn test_out_err() {
        unsafe {
            let popen = Popen::arg("rustc src/run.rs && ./run").exec().unwrap();
            let mut buf = [0 as u8; 4096];
            let mut p;
            while {
                p = fgets(buf.as_mut_ptr() as *mut i8, 4096, popen.stdout);
                p != std::ptr::null_mut::<i8>() && *p != '\0' as i8
            } {
                assert!(strlen(p) != 0);
            }
            println!("");
            while {
                p = fgets(buf.as_mut_ptr() as *mut i8, 4096, popen.stderr);
                p != std::ptr::null_mut::<i8>() && *p != '\0' as i8
            } {
                assert!(strlen(p) != 0);
            }
        }
    }

    #[test]
    fn tty_shell() {}
}
