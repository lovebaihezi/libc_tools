pub struct SocketPair {
    pub io: [libc::c_int; 2],
}

pub enum SocketPairError {
    SocketErrno(libc::c_int),
}

impl SocketPair {
    fn socket_pair(
        domain: libc::c_int,
        type_: libc::c_int,
        protocol: libc::c_int,
    ) -> Result<SocketPair, SocketPairError> {
        let mut sv = [0 as libc::c_int; 2];
        unsafe {
            let result = libc::socketpair(domain, type_, protocol, &mut sv as *mut i32);
            match result {
                -1 => Err(SocketPairError::SocketErrno(*libc::__errno_location())),
                0 => Ok(SocketPair { io: sv }),
                _ => panic!("socket pair: should not reached!"),
            }
        }
    }
}
