use std::io::*;
fn main() {
    let mut buf = [0 as u8; 4096];
    let read_size = stdin().read(&mut buf).unwrap();
    for i in 0..read_size {
        print!("{}", buf[i]);
    }
    println!("");
    eprintln!("read {} size successfully", read_size);
}
