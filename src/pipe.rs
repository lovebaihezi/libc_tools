use std::fmt::Display;

use crate::close::Close;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pipe {
    pub for_read: libc::c_int,
    pub for_write: libc::c_int,
}

impl Pipe {
    pub fn new(&fds: &[libc::c_int; 2]) -> Pipe {
        Pipe {
            for_read: fds[0],
            for_write: fds[1],
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

    pub fn close_read(&mut self) -> Result<(), Close> {
        if self.for_read != -1 {
            Close::close(self.for_read)?;
            self.for_read = -1;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn close_write(&mut self) -> Result<(), Close> {
        if self.for_write != -1 {
            Close::close(self.for_write)?;
            self.for_write = -1;
            Ok(())
        } else {
            Ok(())
        }
    }
}

impl Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            std::format!(
                "pipe: | for_read(fd[0]) {} <<= for_write(fd[1]) {} |",
                self.for_read,
                self.for_write,
            )
            .as_str(),
        )
    }
}

impl Default for Pipe {
    fn default() -> Self {
        Pipe {
            for_read: -1,
            for_write: -1,
        }
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        self.close_read().unwrap();
        self.close_read().unwrap();
    }
}

#[cfg(test)]
mod test {
    use crate::{
        fork::{Fork, ForkPid},
        pipe::Pipe,
        wait::Wait,
    };
    #[test]
    fn test_pipe1() {
        let mut pipe = Pipe::pipe().unwrap();
        match Fork::fork() {
            ForkPid::Parent(_) => {
                pipe.close_write().unwrap();
                Wait::children().unwrap();
                let mut buf = [0 as i8; 4096];
                while unsafe {
                    libc::read(pipe.for_read, buf.as_mut_ptr() as *mut libc::c_void, 4096) > 0
                } {
                    unsafe {
                        assert!(libc::strlen(buf.as_ptr()) != 0);
                    }
                }
            }
            ForkPid::Children(_) => {
                pipe.close_read().unwrap();
                let str = "test pipe\0";
                unsafe {
                    libc::write(
                        pipe.for_write,
                        str.as_ptr() as *const libc::c_void,
                        str.len(),
                    )
                };
            }
            ForkPid::None => panic!(""),
        }
    }

    #[test]
    fn test_pipe2() {
        let mut io_in = Pipe::pipe().unwrap();
        let mut io_out = Pipe::pipe().unwrap();
        match Fork::fork() {
            ForkPid::Parent(_) => {
                io_in.close_write().unwrap();
                let mut buf = [0 as i8; 4096];
                while unsafe {
                    libc::read(io_out.for_read, buf.as_mut_ptr() as *mut libc::c_void, 4096) > 0
                } {
                    unsafe {
                        assert!(libc::strlen(buf.as_ptr()) != 0);
                    }
                }
            }
            ForkPid::Children(_) => {
                io_out.close_read().unwrap();
                let str = "test pipe\0";
                unsafe {
                    libc::write(
                        io_out.for_write,
                        str.as_ptr() as *const libc::c_void,
                        str.len(),
                    )
                };
            }
            ForkPid::None => panic!(""),
        }
    }
}
