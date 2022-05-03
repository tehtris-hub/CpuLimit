use std::env;

use cpulimit::ChildrenMode;
use cpulimit::Pid;

fn main() {
    println!("cpulimit test");

    let mut args = env::args();

    let pid: u32 = args
        .nth(1)
        .expect("No pid found")
        .parse()
        .expect("Error while parsing first arg");

    let pid = Pid::from(pid);
    pid.limit(10.0, ChildrenMode::Exclude)
}
