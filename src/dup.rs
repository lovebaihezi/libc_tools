use std::fmt::Display;

use libc::__errno_location;

pub struct Dup {}

impl Dup {
    pub fn dup(old_fd: libc::c_int) -> Result<libc::c_int, DupError> {
        unsafe {
            let new_fd = libc::dup(old_fd);
            if new_fd == -1 {
                Err(DupError::Errno(*__errno_location()))
            } else {
                Ok(new_fd)
            }
        }
    }
    pub fn dup2(old_fd: libc::c_int, new_fd: libc::c_int) -> Result<libc::c_int, DupError> {
        unsafe {
            let fd = libc::dup2(old_fd, new_fd);
            if fd == -1 || fd != new_fd {
                Err(DupError::Errno(*__errno_location()))
            } else {
                Ok(fd)
            }
        }
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
