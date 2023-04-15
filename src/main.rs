use std::fs;
use log::info;
use smt2parser::{CommandStream, concrete};

use rusmt::context::Context;
use rusmt::solver::{CDCLSolver, SATSolver};

fn main() {
    pretty_env_logger::init();
    let input = fs::read("test.smtlib").unwrap();
    let stream = CommandStream::new(
        &input[..],
        concrete::SyntaxBuilder,
        Some("optional/path/to/file".to_string()),
    );
    let mut context = Context::default();
    let commands = stream.collect::<Result<Vec<_>, _>>().unwrap();
    context.process_commands(commands);
    let mut sat_solver = SATSolver::new(context.get_clauses());

    info!("res: {:?}", sat_solver.solve());
    info!("assignment = {:?}",
        sat_solver.get_assignments()
    );
    let mut cdcl_solver = CDCLSolver::new(context.get_clauses());

    info!("res: {:?}", cdcl_solver.solve());
    info!("assignment = {:?}",
        cdcl_solver.get_assignments()
    );
//     assert!(matches!(commands[..], [
//     concrete::Command::Echo {..},
//     concrete::Command::Exit,
// ]));
//     assert_eq!(commands[0].to_string(), "(echo \"Hello world!\")");
}
