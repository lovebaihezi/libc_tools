use std::rc::Rc;

use crate::{
    dup::{Dup, DupError},
    fork::{Fork, ForkPid},
    pipe::Pipe,
    wait::Wait,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Popen {
    arg: String,
    pid: Option<libc::c_int>,
    fds: Option<[libc::c_int; 3]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PopenError {
    PipeCreateFailed,
    ExecArgFailed(libc::c_int),
    ForkFailed,
    PipeRedirectFailed(libc::c_int),
    Dup2Errno(DupError),
    FdOpenErrno(libc::c_int),
    SocketPairError(libc::c_int),
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
            Self::SocketPairError(v) => {
                std::format!("create socket pair failed! errno: {}", v)
            }
        }
    }
}

impl Popen {
    fn create(arg: &str) -> Box<Popen> {
        Box::new(Popen {
            arg: std::format!("{}\0", arg),
            pid: None,
            fds: None,
        })
    }
    // fn run(self: Box<Popen>) -> Result<Box<Popen>, PopenError> {}
    fn fds(self: Box<Popen>) -> Rc<Box<Popen>> {
        Rc::new(self)
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
    fn testing() {
        let p = Popen::arg("echo");
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
