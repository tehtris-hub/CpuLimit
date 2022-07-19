//! Limit the CPU usage of a process.
//!
//! # Usage
//!
//! Limit process `4562` to 10%.
//!
//! ```console
//! cpulimit --pid 4562 --limit 10
//! ```
//!
//! Run `cpulimit --help` to list all the available options.

use clap::Parser;

use cpulimiter::Pid;

#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    #[clap(
        short,
        long,
        parse(try_from_str),
        help = "The PID of the target process"
    )]
    pid: Pid,
    #[clap(short, long, help = "The CPU rate limit to enforce")]
    limit: f64,
    #[clap(short = 'i', long, help = "Also limit the CPU usage of the children")]
    include_children: bool,
}

fn main() {
    let args = Args::parse();

    if args.include_children {
        args.pid.limit_with_children(args.limit);
    } else {
        args.pid.limit(args.limit);
    };
}
