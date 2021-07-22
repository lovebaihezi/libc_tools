# libc_tools

this crate is build for unix like server

and it aim to get the stdin, stdout, stderr

from exec, (yes, a poor fork to std::process::Command)

it provides three c FILE pointer(stream)

**how to use**
```rust
unsafe {
    let popen = Popen::arg("rustc src/run.rs && ./run").exec().unwrap();
    let mut buf = [0 as u8; 4096];
    let mut p;
    while {
        p = fgets(buf.as_mut_ptr() as *mut i8, 4096, popen.stdout);
        p != std::ptr::null_mut::<i8>() && *p != '\0' as i8
    } {
        assert!(strlen(p) != 0);
    }
    println!("");
    while {
        p = fgets(buf.as_mut_ptr() as *mut i8, 4096, popen.stderr);
        p != std::ptr::null_mut::<i8>() && *p != '\0' as i8
    } {
        assert!(strlen(p) != 0);
    }
}
```
