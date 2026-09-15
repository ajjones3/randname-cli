mod cli;
mod namegen;

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let config = match cli::parse_args(&args) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!(
                "usage: randname [--count N] [--syllables N] [--seed N] [--style common|elvish|dwarvish]"
            );
            std::process::exit(1);
        }
    };

    let seed = config.seed.unwrap_or_else(default_seed);
    let names = namegen::generate_batch(seed, config.count, config.syllables, config.style);

    for name in names {
        println!("{name}");
    }
}

// Reads the system clock and the process id, so it can't be pure and
// stays out of namegen.rs. Everything downstream of the seed it
// produces is deterministic and tested separately.
fn default_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos ^ (std::process::id() as u64)
}
