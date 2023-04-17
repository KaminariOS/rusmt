use std::fs;
use log::info;
use smt2parser::{CommandStream, concrete};
use rusmt::cli::{Cli, Solver};

use clap::Parser;
use rusmt::context::Context;
use rusmt::solver::{CDCLSolver, SATSolver};

fn main() {
    pretty_env_logger::init();
    let args = Cli::parse();
    let input = fs::read(args.path).unwrap();
    let stream = CommandStream::new(
        &input[..],
        concrete::SyntaxBuilder,
        Some("optional/path/to/file".to_string()),
    );
    let mut context = Context::default();
    let commands = stream.collect::<Result<Vec<_>, _>>().unwrap();
    context.process_commands(commands);
    println!("Using solver: {}", args.solver.as_ref());
    match args.solver {
        Solver::BRUTE => {
            let mut sat_solver = SATSolver::new(context.get_clauses());
            println!("res: {}", sat_solver.solve());
            // info!("assignment = {:?};", sat_solver.get_assignments()
            // );
        }
        Solver::CDCL => {
            let mut cdcl_solver = CDCLSolver::new(context.get_clauses());

            println!("{}", cdcl_solver.solve());
            info!("Clauses length: {}", cdcl_solver.clauses.len());
            // info!("assignment = {:?}",
    //         cdcl_solver.get_assignments()
    // );
        }
    }

//     assert!(matches!(commands[..], [
//     concrete::Command::Echo {..},
//     concrete::Command::Exit,
// ]));
//     assert_eq!(commands[0].to_string(), "(echo \"Hello world!\")");
}
