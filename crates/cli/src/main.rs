#![feature(test)]

extern crate test;

mod puzzles;

use crate::puzzles::*;
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

#[cfg(test)]
mod tests {
    use super::*;
    use test::{black_box, Bencher};

    fn solve(puzzle: [u8; 81]) {
        let input = Board { inner: puzzle };
        let mut solver = Solver::default();

        let board = solver.solve(input);

        black_box(&board);
    }

    #[bench]
    fn bench_puzzle1(b: &mut Bencher) {
        b.iter(|| {
            solve(INPUT1);
        });
    }

    #[bench]
    fn bench_puzzle2(b: &mut Bencher) {
        b.iter(|| {
            solve(INPUT2);
        });
    }

    #[bench]
    fn bench_puzzle3(b: &mut Bencher) {
        b.iter(|| {
            solve(INPUT3);
        });
    }
}
