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

#[cfg(test)]
mod tests {
    use super::*;

    fn test(input: [u8; 81], output: [u8; 81]) {
        let input = Board { inner: input };
        let board = solve(input, false);
        assert!(board.inner == output);
    }

    #[test]
    fn test_puzzle1() {
        test(INPUT1, OUTPUT1);
    }
    #[test]
    fn test_puzzle2() {
        test(INPUT2, OUTPUT2);
    }
    #[test]
    fn test_puzzle3() {
        test(INPUT3, OUTPUT3);
    }
}
