use akari::akari::Akari;
use std::{io, io::BufRead, time::Instant};
use z3::SatResult;

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
    // Get stdin.
    let board = get_std_in();

    // Construct game from stdin.
    let mut game = Akari::from(board);

    println!("problem\n{}", game);

    // Construct context.
    let context = z3::Context::new(&z3::Config::default());

    // Create asserts.
    let asserts = game.get_asserts(&context);

    // Construct solver.
    let solver = z3::Solver::new(&context);

    for assert in asserts.iter() {
        solver.assert(assert);
    }

    let start = Instant::now();

    match solver.check() {
        SatResult::Sat => {
            println!("Sat\n");
            if let Some(model) = solver.get_model() {
                // println!("{:?}", model);
                game.set_solution(&context, model);
                println!("solution found in {:?}\n{}", start.elapsed(), game);
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
