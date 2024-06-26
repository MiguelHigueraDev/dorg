use std::env;
use std::process;

use dorg::Config;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {err}");
        process::exit(1);
    });
    
    if let Err(e) = dorg::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
