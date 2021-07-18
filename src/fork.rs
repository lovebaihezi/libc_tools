#[derive(Debug, Clone)]
pub enum ForkPid {
    Parent(i32),
    Children(i32),
}

#[derive(Debug, Default, Clone)]
pub struct Fork {
    parent: Option<i32>,
    children: Option<i32>,
}

impl Fork {
    pub fn create(&mut self) -> Option<ForkPid> {
        let pid = unsafe { libc::fork() };
        match pid {
            0 => {
                let children_pid = unsafe { libc::getpid() };
                self.children = Some(children_pid);
                Some(ForkPid::Children(children_pid))
            }
            1..=std::i32::MAX => {
                self.parent = Some(pid);
                Some(ForkPid::Parent(pid))
            }
            _ => None,
        }
    }
    pub fn fork() -> Option<ForkPid> {
        let pid = unsafe { libc::fork() };
        match pid {
            0 => Some(ForkPid::Children(unsafe { libc::getpid() })),
            1..=std::i32::MAX => Some(ForkPid::Parent(pid)),
            _ => None,
        }
    }
    pub fn new() -> Fork {
        Fork {
            parent: None,
            children: None,
        }
    }
}
