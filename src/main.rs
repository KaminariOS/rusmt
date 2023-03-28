use std::fs;
use smt2parser::{CommandStream, concrete};


use rusmt::Solver;

fn main() {
    let input = fs::read("test.smtlib").unwrap();
    let stream = CommandStream::new(
        &input[..],
        concrete::SyntaxBuilder,
        Some("optional/path/to/file".to_string()),
    );
    let mut solver = Solver::default();
    let commands = stream.collect::<Result<Vec<_>, _>>().unwrap();
    solver.process_commands(commands);
//     assert!(matches!(commands[..], [
//     concrete::Command::Echo {..},
//     concrete::Command::Exit,
// ]));
//     assert_eq!(commands[0].to_string(), "(echo \"Hello world!\")");
}
