#[allow(unused_imports)]
use ergo::*;
#[allow(unused_imports)]
use quicli::prelude::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "ls", about = "List and filter artifacts")]
pub struct Ls {
    /// Pass many times for more log output.
    #[structopt(long = "verbose", short = "v")]
    pub verbosity: u64,

    #[structopt(help = "Pattern to search for")]
    pub pattern: String,
}

pub fn run(cmd: Ls) -> Result<i32> {
    set_log_verbosity(cmd.verbosity)?;
    Ok(0)
}
