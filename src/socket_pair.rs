use std::fmt::Display;
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]

pub struct SocketPair {
    pub sv: [libc::c_int; 2],
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]

pub enum SocketPairError {
    SocketErrno(libc::c_int),
}

impl Display for SocketPairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Self::SocketErrno(v) => {
                f.write_str(format!("create socket with socket pair failed! errno: {}", v).as_str())
            }
        }
    }
}

impl SocketPair {
    fn create(
        domain: libc::c_int,
        type_: libc::c_int,
        protocol: libc::c_int,
    ) -> Result<SocketPair, SocketPairError> {
        let mut sv = [0 as libc::c_int; 2];
        unsafe {
            let result = libc::socketpair(domain, type_, protocol, &mut sv as *mut i32);
            match result {
                -1 => Err(SocketPairError::SocketErrno(*libc::__errno_location())),
                0 => Ok(SocketPair { sv }),
                _ => panic!("socket pair: should not reached!"),
            }
        }
    }
}
