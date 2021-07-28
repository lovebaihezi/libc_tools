use std::fmt::Display;

#[derive(Debug)]
pub enum Wait {
    WaitFailure(i32),
    WNoHangExit,
    ErrnoNotFound,
}

impl Display for Wait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Wait::ErrnoNotFound => "errno not found!",
            _ => "exec failed!",
        })
    }
}

impl Wait {
    pub fn children() -> Result<(libc::pid_t, libc::c_int), Wait> {
        let mut status = 0;
        let result = unsafe { libc::wait(&mut status as *mut libc::c_int) };
        let errno = unsafe { libc::__errno_location() };
        if errno == std::ptr::null_mut::<libc::c_int>() {
            Err(Wait::ErrnoNotFound)
        } else {
            match result {
                -1 => Err(Wait::WaitFailure(unsafe { *errno })),
                _ => Ok((result, status)),
            }
        }
    }
    pub fn  children_with(
        pid: libc::pid_t,
        options: libc::c_int,
    ) -> Result<(libc::pid_t, libc::c_int), Wait> {
        let mut w_status = 0;
        let result = unsafe { libc::waitpid(pid, &mut w_status as *mut libc::c_int, options) };
        let errno = unsafe { libc::__errno_location() };
        if errno == std::ptr::null_mut::<libc::c_int>() {
            Err(Wait::ErrnoNotFound)
        } else {
            match result {
                0 if options & libc::WNOHANG != 0 => Err(Wait::WNoHangExit),
                -1 => Err(Wait::WaitFailure(unsafe { *errno })),
                _ => Ok((result, w_status)),
            }
        }
    }
}
