use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExperimentError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Input configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: PathBuf,

    /// List of parameters to override
    #[arg(short, long, value_delimiter = ',')]
    pub params: Option<Vec<String>>,

    /// Number of iterations
    #[arg(short, long, default_value_t = 1)]
    pub iterations: u32,

    /// Subcommand
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run simulation
    Simulate {
        /// Simulation time (seconds)
        #[arg(short, long, default_value_t = 60.0)]
        duration: f64,
    },
    /// Generate report
    Report {
        /// Output format
        #[arg(short, long, default_value = "html")]
        format: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExperimentConfig {
    pub name: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Parameter {
    pub name: String,
    pub value: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

pub fn load_config(path: &Path) -> Result<ExperimentConfig, ExperimentError> {
    let file = std::fs::File::open(path)
        .map_err(|_| ExperimentError::FileNotFound(path.to_path_buf()))?;
    Ok(serde_yaml::from_reader(file)?)
}

pub fn run_experiment(args: Args) -> Result<(), ExperimentError> {
    let config = load_config(&args.config)?;
    
    println!("Running experiment: {}", config.name);
    println!("Parameters:");
    for param in config.parameters {
        println!("- {}: {}", param.name, param.value);
    }

    if let Some(params) = args.params {
        println!("Overridden parameters: {:?}", params);
    }

    println!("Iterations: {}", args.iterations);

    match args.command {
        Some(Commands::Simulate { duration }) => {
            println!("Simulating for {} seconds", duration);
        }
        Some(Commands::Report { format }) => {
            println!("Generating report in {} format", format);
        }
        None => {
            println!("No subcommand specified");
        }
    }

    Ok(())
}