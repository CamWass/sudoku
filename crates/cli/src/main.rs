use data::*;
use solver::{Board, Solver};

fn main() {
    let input = Board { inner: INPUT3 };
    let mut solver = Solver::default();

    println!("Input:");
    input.print();

    let board = solver.solve(input);

    println!("Output:");
    board.print();

    if board.is_solved() {
        println!("Solved puzzle");
    } else {
        let old_empty_squares = input.empty_squares();
        let new_empty_squares = board.empty_squares();

        let placed = old_empty_squares - new_empty_squares;

        println!(
            "Could not solve puzzle. {} empty squares remaining (placed {})",
            new_empty_squares, placed
        );
    }

    debug_assert!(board.inner == OUTPUT3);
}
