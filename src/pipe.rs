use crate::Close;
use libc::c_int;
use std::fmt::Display;

pub enum PipeSide {
    Read,
    Write,
}

// the other side of the pipe need to closed when you need to use one side, or one process will be suspend
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pipe {
    fd: (c_int, c_int),
}

impl Pipe {
    pub fn new(&fds: &[libc::c_int; 2]) -> Pipe {
        Pipe {
            fd: (fds[0], fds[1]),
        }
    }
    pub fn pipe() -> Option<Pipe> {
        let mut end = [0; 2];
        match unsafe { libc::pipe(end.as_mut_ptr()) } {
            libc::INT_MIN..=-1 => None,
            _ => Some(Pipe::new(&end)),
        }
    }

    pub fn pipe2(flag: i32) -> Option<Pipe> {
        let mut end = [0; 2];
        match unsafe { libc::pipe2(end.as_mut_ptr(), flag) } {
            libc::INT_MIN..=-1 => None,
            _ => Some(Pipe::new(&end)),
        }
    }

    pub fn get(&self, side: PipeSide) -> c_int {
        match side {
            PipeSide::Read => self.fd.1,
            PipeSide::Write => self.fd.0,
        }
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        match Close::close(&[self.fd.0, self.fd.1]) {
            Ok(_) => (),
            Err(v) => eprintln!("{}", v),
        }
    }
}

impl Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            std::format!(
                "pipe: | for_read(fd[0]) {} <<= for_write(fd[1]) {} |",
                self.fd.0,
                self.fd.1,
            )
            .as_str(),
        )
    }
}
#[macro_export]
macro_rules! create_pipe {
    (1) => {{
        let mut pipe = [libc::c_int; 2];
        match unsafe {
            pipe(pipe.as_mut_ptr())
        } {
            -1 => None,
            _  => Some(pipe)
        }
    }};
    ($n: expr) => {{
        if $n <= 0 {
            None
        } else {
            let mut pipes: [[libc::c_int; 2]; $n] = [[0; 2]; $n];
            let mut success = true;
            for i in 0..$n as usize {
                unsafe {
                    match libc::pipe(pipes[i].as_mut_ptr()) {
                        0 => {}
                        _ => {
                            success = false;
                            break;
                        }
                    }
                };
            }
            if success {
                Some(pipes)
            } else {
                None
            }
        }
    }};
}

#[macro_export]
macro_rules! create_pipe2 {
    ($n: expr, $modes: expr) => {{
        if $n != $modes.len() || $n == 0 {
            None
        } else {
            let mut pipes: [[libc::c_int; 2]; $n] = [[0; 2]; $n];
            let mut success = true;
            for i in 0..$n as usize {
                unsafe {
                    match libc::pipe2(pipes[i].as_mut_ptr(), $modes[i]) {
                        0 => {}
                        _ => {
                            success = false;
                            break;
                        }
                    }
                };
            }
            if success {
                Some(pipes)
            } else {
                None
            }
        }
    }};
}

#[cfg(test)]
mod pipe {
    use std::{error::Error, ffi::CString};

    use libc::execl;

    use crate::Close;

    #[test]
    fn test_execl() -> Result<(), Box<dyn Error>> {
        let path = CString::new("/bin/sh")?;
        let sh = CString::new("sh")?;
        let exec = CString::new("-c")?;
        let arg = CString::new("ls")?;
        // unsafe { execl(path.as_ptr(), sh.as_ptr(), exec.as_ptr(), arg.as_ptr(), 0) };
        Ok(())
    }

    #[test]
    fn test_create_pipe_macro() {
        let pipes1 = create_pipe!(1 + 1).unwrap();
        let pipes2 = create_pipe!(1 & 1).unwrap();
        assert!(pipes1.len() == 2);
        assert!(pipes2.len() == 1);
        let s = &pipes1.iter().flatten().map(|x| *x).collect::<Vec<i32>>()[..];
        Close::close(s).unwrap();
        let s1 = &pipes2.iter().flatten().map(|x| *x).collect::<Vec<i32>>()[..];
        Close::close(s1).unwrap();
    }
}
