use libc::fgets;
use libc_tools::Popen;

fn main() {
    let popen = Popen::arg("time")
        .exec()
        .unwrap();
    unsafe {
        let mut buf = [0 as i8; 4096];
        let mut p;
        let mut i = 0;
        while {
            p = fgets(buf.as_mut_ptr(), 4096, popen.stdout);
            p != std::ptr::null_mut::<i8>()
        } {
            i += 1;
            if i > 10 {
                break;
            }
        }
    }
}
