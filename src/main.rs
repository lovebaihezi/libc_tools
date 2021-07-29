// use std::{error::Error, ptr::null_mut};

// use libc::*;
// use libc_tools::Pty;

// fn main() -> Result<(), Box<dyn Error>> {
//     let pty = Pty::new(null_mut::<termios>(), null_mut::<winsize>()).unwrap();
//     println!(
//         "{}",
//         match &pty.device_name {
//             Some(v) => v.clone(),
//             None => panic!(""),
//         }
//     );
//     unsafe {
//         write(
//             pty.pty_fd.unwrap(),
//             "date \n\0".as_ptr() as *const c_void,
//             7,
//         )
//     };
//     let mut buf = [0 as u8; 4096];
//     let mut read_size;
//     let mut index = 0;
//     while unsafe {
//         read_size = read(pty.pty_fd.unwrap(), buf.as_mut_ptr() as *mut c_void, 4096);
//         read_size != 0 && read_size != -1 && index <= 3
//     } {
//         index += 1;
//         assert!(read_size != -1 && read_size != 0);
//         for i in &buf[..read_size as usize] {
//             print!("{}", *i as char);
//         }
//         println!("");
//     }
//     for i in &buf[..read_size as usize] {
//         print!("{}", *i as char);
//     }
//     println!("");
//     // pty.drop()?;
//     Ok(())
// }
