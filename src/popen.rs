use crate::{
    dup::{Dup, DupError},
    fork::{Fork, ForkPid},
    pipe::Pipe,
    wait::Wait,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Popen {
    pub arg: String,
    pub io: Option<[libc::c_int; 3]>,
    pub pid: Option<libc::c_int>,
    fds: Option<[Pipe; 3]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PopenError {
    PipeCreateFailed,
    ExecArgFailed(libc::c_int),
    ForkFailed,
    PipeRedirectFailed(libc::c_int),
    Dup2Errno(DupError),
    FdOpenErrno(libc::c_int),
}

impl PopenError {
    fn to_string(&self) -> String {
        match *self {
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
        }
    }
}
#[deprecated(note = "do not use!")]
impl Popen {
    fn arg(arg: &str) -> Box<Popen> {
        Box::new(Popen {
            arg: String::from(arg),
            io: None,
            pid: None,
            fds: None,
        })
    }

    fn run(mut self: Box<Popen>) -> Result<Box<Popen>, PopenError> {
        let fds = Some(
            match [Pipe::pipe(), Pipe::pipe(), Pipe::pipe()] {
                [Some(v1), Some(v2), Some(v3)] => Some([v1, v2, v3]),
                _ => None,
            }
            .ok_or(PopenError::PipeCreateFailed)?,
        );
        match Fork::fork() {
            ForkPid::Parent((_, children)) => {
                self.pid = Some(children);
                match &fds {
                    Some([io_in, io_out, io_err]) => {
                        self.io = Some([io_in.for_write, io_out.for_read, io_err.for_read]);
                        self.fds = fds;
                        Ok(self)
                    }
                    None => Err(PopenError::PipeCreateFailed),
                }
            }
            ForkPid::Children(_) => match &fds {
                Some([v1, v2, v3]) => {
                    unsafe {
                        libc::close(v1.for_write);
                        libc::close(v2.for_read);
                        libc::close(v3.for_read);
                    }
                    let dup = (
                        Dup::dup2(v1.for_read, libc::STDIN_FILENO),
                        Dup::dup2(v2.for_write, libc::STDOUT_FILENO),
                        Dup::dup2(v3.for_write, libc::STDERR_FILENO),
                    );
                    match dup {
                        (Err(v), _, _) | (_, Err(v), _) | (_, _, Err(v)) => {
                            Err(PopenError::Dup2Errno(v))
                        }
                        _ => {
                            // unsafe {
                            //     libc::close(v1.for_read);
                            //     libc::close(v2.for_write);
                            //     libc::close(v3.for_write);
                            // };
                            Err(PopenError::ExecArgFailed(unsafe {
                                libc::execl(
                                    "/bin/sh\0".as_ptr() as *const libc::c_char,
                                    "-c\0".as_ptr() as *const libc::c_char,
                                    self.arg.as_ptr() as *const libc::c_char,
                                    std::ptr::null::<libc::c_char>(),
                                )
                            }))
                            // panic!("exec arg failed!");
                        }
                    }
                }
                None => Err(PopenError::PipeCreateFailed),
            },
            ForkPid::None => Err(PopenError::ForkFailed),
        }
    }
}

impl Drop for Popen {
    fn drop(&mut self) {
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
mod test {
    use super::Popen;

    #[test]
    fn popen_echo() {
        unsafe {
            let echo = Popen::arg("ls").run().unwrap();
            let buf = [0 as i8; 4096];
            match &echo.io {
                Some([v1, v2, v3]) => {
                    let buf = [0 as i8; 4096];
                    let mut read_size = 0;
                    while {
                        read_size = libc::read(*v2, buf.as_ptr() as *mut libc::c_void, 4096);
                        eprintln!("{}", read_size);
                        read_size >= 0
                    } {
                        eprintln!("{}", read_size);
                        libc::perror(buf.as_ptr());
                        assert!(libc::strlen(buf.as_ptr()) != 0);
                    }
                }
                None => panic!("popen create failed!"),
            }
        }
    }

    #[test]
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
                libc::perror(buf.as_ptr());
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
}
