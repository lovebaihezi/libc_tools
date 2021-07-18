#[derive(Debug)]
enum Wait {
    WaitFailure,
    WNoHangExit,
}
impl Wait {
    pub fn children() -> Result<(libc::pid_t, libc::c_int), Wait> {
        let mut status = 0;
        let result = unsafe { libc::wait(&mut status as *mut i32) };
        match result {
            -1 => Err(Wait::WaitFailure),
            _ => Ok((result, status)),
        }
    }
    pub fn children_with(
        pid: libc::pid_t,
        options: libc::c_int,
    ) -> Result<(libc::pid_t, libc::c_int), Wait> {
        let mut w_status = 0;
        let result = unsafe { libc::waitpid(pid, &mut w_status as *mut libc::c_int, options) };
        match result {
            0 if options & libc::WNOHANG != 0 => Err(Wait::WNoHangExit),
            -1 => Err(Wait::WaitFailure),
            _ => Ok((result, w_status)),
        }
    }
}
