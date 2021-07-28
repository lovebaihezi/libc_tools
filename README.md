# libc_tools

**this crate wrapper some sys_call in libc**

thanks to libc
this crate has some wrapper for libc:
fork
eg:
``` rust
match Fork::fork() {
    ForkPid::Parent((parent, children)) => {}
    ForkPid::Children((parent, children)) => {}
    ForkPid::None => panic!("fork failed!")
}
```
forkpt
dup
dup2(and dup2s for mutliply fd)

``` rust
let x : () = Dup::dup2s(&[...olds], &[...news]).unwrap();
```

popen
**how to use**
```rust
unsafe {
    let popen = Popen::arg("date").exec().unwrap();
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
