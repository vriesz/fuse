// src/main.rs

mod experiment;

use clap::Parser;
use experiment::{Args, run_experiment};

fn main() {
    match run_experiment(Args::parse()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}