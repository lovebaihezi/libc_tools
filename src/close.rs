use std::fmt::Display;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Close {
    Errno(libc::c_int),
}

impl Display for Close {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Close::Errno(v) => f.write_str(std::format!("close fd failed! errno: {}", v).as_str()),
        }
    }
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
