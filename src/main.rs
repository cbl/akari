use clap::Parser;
use itertools::Itertools;
use numberlink::akari::Akari;
use z3::SatResult;
use std::{io, io::BufRead};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Whether to pretty print the output.
    #[clap(short, long)]
    pretty: bool,
}

fn get_std_in() -> String {
    let stdin = io::stdin();
    let mut input = String::new();

    for line in stdin.lock().lines() {
        input += &line.unwrap();
        input += &"\n".to_string();
    }

    input
}

fn main() {
    // Parse cli arguments.
    let args = Args::parse();

    // Get stdin.
    let board = get_std_in();

    // Construct game from stdin.
    let mut game = Akari::from(board);

    println!("{}", game);

    // Construct context.
    let context = z3::Context::new(&z3::Config::default());

    // Create asserts.
    let asserts = game.get_asserts(&context);

    // Construct solver.
    let solver = z3::Solver::new(&context);

    for assert in asserts.iter() {
        solver.assert(assert);
    }

    match solver.check() {
        SatResult::Sat => {
            println!("Sat");
            if let Some(model) = solver.get_model() {
                println!("{:?}", model);
                game.set_solution(&context, model);
                println!("{}", game);
            }
        }
        SatResult::Unknown => {
            println!("Unknown");
        }
        SatResult::Unsat => {
            println!("Unsat");
        }
    }
}
