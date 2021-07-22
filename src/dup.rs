use std::fmt::Display;

use libc::{__errno_location, c_int};

use crate::Close;

pub struct Dup {
    fd: libc::c_int,
}

impl Dup {
    pub fn dup(old_fd: libc::c_int) -> Result<libc::c_int, DupError> {
        unsafe {
            match libc::dup(old_fd) {
                -1 => Err(DupError::Errno(*__errno_location())),
                fd @ _ => Ok(fd),
            }
        }
    }
    pub fn dup2(old_fd: libc::c_int, new_fd: libc::c_int) -> Result<libc::c_int, DupError> {
        unsafe {
            match libc::dup2(old_fd, new_fd) {
                fd if fd == new_fd => Ok(fd),
                -1 => Err(DupError::Errno(*__errno_location())),
                _ => panic!("this should not reached!"),
            }
        }
    }
    pub fn transform(fd: libc::c_int) -> Dup {
        Dup { fd }
    }

    pub fn close_to(&self, fd: libc::c_int) -> Result<libc::c_int, DupError> {
        if self.fd != fd {
            let result = Dup::dup2(self.fd, fd);
            result
        } else {
            Ok(fd)
        }
    }

    pub fn dup2s<'a, 'b>(old_fds: &'a [c_int], new_fds: &'b [c_int]) -> Result<(), DupError> {
        if old_fds.len() != new_fds.len() {
            panic!("dup2s: old fd array length should equal to new fds array");
        }
        for i in 0..old_fds.len() {
            Self::transform(old_fds[i]).close_to(new_fds[i])?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DupError {
    Errno(libc::c_int),
}

impl Display for DupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &DupError::Errno(v) => f.write_str(std::format!("dup error, errno: {}", v).as_str()),
        }
    }
}
