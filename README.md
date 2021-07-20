# libc_tools

this crate is build for unix like server

and it aim to get the stdin, stdout, stderr

from exec, (yes, a poor fork to std::process::Command)

it provides three c FILE pointer(stream)

**how to use**
```rust
let Popen { stdin, stdout, stderr } = Popen::new("echo hello world");
```
