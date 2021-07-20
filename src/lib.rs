pub mod dup;
pub mod fork;
pub mod pipe;
pub mod popen;
pub mod wait;
pub mod close;
#[cfg(test)]
mod test {

    #[test]
    fn create_fork() {}
}
