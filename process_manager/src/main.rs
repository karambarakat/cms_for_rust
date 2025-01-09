mod commands;


#[cfg(not(feature = "basic_daemon"))]
pub(crate) mod basic_daemon;
#[cfg(feature = "basic_daemon")]
pub mod basic_daemon;

fn main() {
    if cfg!(feature = "basic_daemon") {
        basic_daemon::main();
    } else {
        panic!("a api for deamon is not available yet, please enable the feature 'basic_daemon'");
    }
}
