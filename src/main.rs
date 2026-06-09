mod cli;
mod config;
mod error;
mod launcher;
mod paths;
mod stacker;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
