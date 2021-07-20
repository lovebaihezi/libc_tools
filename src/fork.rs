#[derive(Debug, Clone)]
pub enum ForkPid {
    Parent((i32, i32)),
    Children((i32, i32)),
    None,
}

#[derive(Debug, Default, Clone)]
pub struct Fork;

impl Fork {
    pub fn fork() -> ForkPid {
        let pid = unsafe { libc::fork() };
        match pid {
            0 => ForkPid::Children((unsafe { libc::getppid() }, unsafe { libc::getpid() })),
            1..=std::i32::MAX => ForkPid::Parent((unsafe { libc::getpid() }, pid)),
            _ => ForkPid::None,
        }
    }
}
