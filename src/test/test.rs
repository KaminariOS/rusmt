use crate::solver::Res;
use crate::test::generator::Generator;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

fn print_duration(time: Instant) -> String {
    let seconds = time.elapsed().as_secs_f32();
    if seconds > 1. {
        format!("time: {} s", seconds)
    } else {
        format!("time: {} ms", 1000. * seconds)
    }
}

#[test]
fn test() {
    if !Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .unwrap()
        .success()
    {
        eprintln!("Build failure.");
    }
    let generator = Generator {
        variables: 2000,
        clauses: 6000,
    };
    let test_file = "random.smtlib";
    generator.generate(test_file);

    let mut logfile = File::create("test.log").unwrap();

    // let start = Instant::now();
    // let output = Command::new("./target/release/rusmt")
    //     .args(["BRUTE", test_file])
    //     .env("RUST_LOG", "info")
    //     .output()
    //     .expect("failed to execute process");
    // let time = print_duration(start);
    // let txt = std::str::from_utf8(&output.stdout).unwrap();
    // let report = format!("{}\n {}", time, txt);
    // println!("{}", report);
    // writeln!(logfile, "{}", report).unwrap();

    let start = Instant::now();
    let output = Command::new("./target/release/rusmt")
        .args(["CDCL", test_file])
        .env("RUST_LOG", "info")
        .output()
        .expect("failed to execute process");
    let time = print_duration(start);
    let output_cdcl = std::str::from_utf8(&output.stdout).unwrap();
    let report = format!("{}\n {}", time, output_cdcl);
    println!("{}", report);
    writeln!(logfile, "{}", report).unwrap();

    let start = Instant::now();
    let output = Command::new("z3")
        .args([test_file])
        .output()
        .expect("failed to execute process");
    let time = print_duration(start);
    let output_z3 = std::str::from_utf8(&output.stdout).unwrap();
    let txt = format!("Using solver: {}\nres: {}", "z3", output_z3);
    let report = format!("{}\n {}", time, txt);
    println!("{}", report);
    writeln!(logfile, "{}", report).unwrap();
    let unsat = Res::UNSAT.as_ref();
    if output_z3.contains(unsat) != output_cdcl.contains(unsat) {
        panic!("Incorrect.")
    }
}
