pub mod dup;
pub mod fork;
pub mod pipe;
pub mod popen;
pub mod wait;
pub mod close;
pub mod proc;
mod run;
pub mod socket_pair;
#[cfg(test)]
mod test {

    #[test]
    fn create_fork() {}
}
