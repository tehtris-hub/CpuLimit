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

use std::process::exit;
use std::thread;
use std::time::Duration;

use clap::Parser;

use cpulimiter::{CpuLimit, Pid};

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

    let limiter = if args.include_children {
        CpuLimit::new_with_children(args.pid, args.limit)
    } else {
        CpuLimit::new(args.pid, args.limit)
    }
    .unwrap();

    ctrlc::set_handler(move || {
        println!("Stopping after receiving Ctrl-C");
        limiter.stop().unwrap();
        // wait for the Stop command to propagate.
        thread::sleep(Duration::from_millis(200));
        exit(0);
    })
    .unwrap();

    loop {
        thread::sleep(Duration::from_secs(1));
        if !args.pid.alive() {
            println!("The target process is dead");
            break;
        }
    }
}
