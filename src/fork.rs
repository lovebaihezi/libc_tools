use libc::{getpid, getppid};

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
            0 => ForkPid::Children((unsafe { getppid() }, unsafe { getpid() })),
            1..=std::i32::MAX => ForkPid::Parent((unsafe { getpid() }, pid)),
            _ => ForkPid::None,
        }
    }
}

#[cfg(test)]
mod fork {

    #[test]
    fn fork_box() {}
}
