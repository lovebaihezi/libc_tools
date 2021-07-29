use std::{
    error::Error,
    ptr::{null, null_mut},
};

use libc::{__errno_location, _exit, c_int, execlp, forkpty, termios, winsize};

use crate::{Close, Wait};

/*
 * a pty terminal will have
 * 1. pty fd
 * 2. windows size
 * 3. terminal attributes
 * 4. device name
 *
 * so to create a pty terminal, you need
 * 1.get config for terminal size and windows size(Optional)
 * 3.forkpty and get pty terminal name(Required), pty fd
 *
 * when it reach its life end, what to do?
 * 1.clean up then fd
 * 2.wait or stop then child process id
 * 3.
 */
#[derive(Debug, Clone)]
pub struct Pty {
    pub pty_fd: Option<c_int>,
    pub pid: Option<c_int>,
    pub device_name: Option<String>,
    pub terminal_attr: *mut termios,
    pub windows_size: *mut winsize,
}

#[derive(Debug, Clone, Copy)]
pub enum PtyError {
    ForkFailed(c_int),
    CreatePtyFailed(c_int),
}

impl Pty {
    pub fn new(terminal_attr: *mut termios, windows_size: *mut winsize) -> Result<Pty, PtyError> {
        let mut pty_fd = 0;
        let mut name = [0 as u8; 50];
        let pid = unsafe {
            forkpty(
                &mut pty_fd as *mut i32,
                name.as_mut_ptr() as *mut i8,
                terminal_attr,
                windows_size,
            )
        };
        let device_name = String::from_utf8_lossy(&name[..]).to_string();
        match pid {
            i32::MIN..=-1 => Err(PtyError::ForkFailed(unsafe { *__errno_location() })),
            0 => unsafe {
                _exit(execlp(
                    "/bin/zsh\0".as_ptr() as *const i8,
                    "-i\0".as_ptr() as *const i8,
                    null::<i8>(),
                ))
            },
            _ => Ok(Pty {
                pty_fd: Some(pty_fd),
                device_name: Some(device_name),
                terminal_attr,
                windows_size,
                pid: Some(pid),
            }),
        }
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        Close::close(&[self.pty_fd.unwrap()]).or_else(|x| Err(format!("{}", x))).unwrap();
        Wait::children_with(self.pid.unwrap(), 0).or_else(|x| Err(format!("{}", x))).unwrap();
    }
}

#[cfg(test)]
mod pty {
    use std::{error::Error, ptr::null_mut};

    use libc::{c_void, read, termios, winsize, write};

    use crate::Pty;

    #[test]
    #[ignore] // cargo test this will not wait for drop, so the terminal will suspend
    fn test_pty() -> Result<(), Box<dyn Error>> {
        let pty = Pty::new(null_mut::<termios>(), null_mut::<winsize>()).unwrap();
        println!(
            "{}",
            match &pty.device_name {
                Some(v) => v.clone(),
                None => panic!(""),
            }
        );
        unsafe {
            write(
                pty.pty_fd.unwrap(),
                "date \n\0".as_ptr() as *const c_void,
                7,
            )
        };
        let mut buf = [0 as u8; 4096];
        let mut read_size;
        let mut index = 0;
        while unsafe {
            read_size = read(pty.pty_fd.unwrap(), buf.as_mut_ptr() as *mut c_void, 4096);
            read_size != 0 && read_size != -1 && index <= 3
        } {
            index += 1;
            assert!(read_size != -1 && read_size != 0);
            for i in &buf[..read_size as usize] {
                print!("{}", *i as char);
            }
            println!("");
        }
        for i in &buf[..read_size as usize] {
            print!("{}", *i as char);
        }
        println!("");
        // pty.drop()?;
        Ok(())
    }
}
