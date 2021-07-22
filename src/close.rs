use libc::{__errno_location, fclose, FILE};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Close {
    CloseErrno(libc::c_int),
    FCloseErrno(libc::c_int),
}

impl Display for Close {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Close::CloseErrno(v) => {
                f.write_str(std::format!("close fd failed! errno: {}", v).as_str())
            }
            Close::FCloseErrno(v) => {
                f.write_str(std::format!("fclose file failed! errno: {}", v).as_str())
            }
        }
    }
}

impl Close {
    pub fn close<'a>(fds: &'a [libc::c_int]) -> Result<(), Close> {
        for i in fds {
            unsafe {
                match libc::close(*i) {
                    -1 => Err(Close::CloseErrno(*libc::__errno_location())),
                    0 => Ok(()),
                    _ => panic!("this should not reached!"),
                }?
            };
        }
        Ok(())
    }

    pub fn fclose<'a>(ps: &'a [*mut FILE]) -> Result<(), Close> {
        unsafe {
            for i in ps {
                match fclose(*i) {
                    0 => Ok(()),
                    -1 => Err(Close::FCloseErrno(*__errno_location())),
                    _ => panic!("this should not reached!"),
                }?
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod close {
    use crate::{Close, Pipe, PipeSide};

    #[test]
    fn test_close_pipe() {
        Pipe::pipe().unwrap();
    }
}
