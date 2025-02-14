use rust_sim::logger::Logger;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: analyze <log_filename>");
        return;
    }

    let filename = &args[1];
    let logger = Logger::new_for_analysis();
    logger.analyze(filename);
}
