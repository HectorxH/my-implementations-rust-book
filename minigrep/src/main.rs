use std::env;
use std::process;

use minigrep::Config;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parasing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = minigrep::run(&config) {
        eprintln!("Problem running the program: {e}");
        process::exit(1);
    };
}
