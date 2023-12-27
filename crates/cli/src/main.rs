use data::*;
use solver::{solve, Board};

#[rustfmt::skip] 
pub static INPUT: [u8; 81] = [
    0,0,0, 0,0,0, 0,0,0,
    0,0,0, 0,0,0, 0,0,0,
    0,0,0, 0,0,0, 6,0,0,

    0,0,0, 0,9,0, 0,0,0,
    0,0,0, 0,0,0, 0,0,0,
    0,0,0, 0,4,0, 0,6,0,

    0,0,0, 0,0,0, 0,0,0,
    0,0,0, 0,0,0, 0,0,0,
    0,0,0, 0,0,0, 0,0,0,
];

fn main() {
    let input = Board { inner: INPUT };

    println!("Input:");
    input.print();

    let board = solve(input, true);

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

    // debug_assert!(board.inner == OUTPUT3);
}
