#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Close {
    Errno(libc::c_int),
}

impl Close {
    pub fn close(fd: libc::c_int) -> Result<(), Close> {
        unsafe {
            match libc::close(fd) {
                -1 => Err(Close::Errno(*libc::__errno_location())),
                0 => Ok(()),
                _ => panic!("this should not reached!"),
            }
        }
    }
}
