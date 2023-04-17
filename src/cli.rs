use clap::Parser;
use strum_macros::*;

/// Simple SAT solver
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// The solver to used
    pub solver: Solver,
    /// The path to the file to read
    pub path: std::path::PathBuf,
}

#[derive(EnumString, AsRefStr)]
pub enum Solver {
    BRUTE,
    CDCL,
}
