use rayon::prelude::*;
use std::{
    hint::unreachable_unchecked,
    sync::atomic::{AtomicBool, Ordering},
};

/*
Pick square
choose number not in row/column/box
continue until contradiction/ no options
backtrack


for each 'group' (row/column/block)
    for each missing number
        gather constraints for each empty square in group
        if only one square's constraints permit the missing number
            place number in square


1. Obvious
2. Speculate on square
3. Obvious until solved
    if trapped, revert speculation and all of its moves, try again (step 2)

By cycling between obvious and speculation, we.... make our
speculations more likely???

If faced with many possible moves, pick them in most likely order.
e.g. if there's a 50% chance a square is a 3, and a 33% chance it's
a 9, choose 3 since it's more likely.

TODO: can we use the GPU to parallelise the search?

*/

#[derive(Clone, Copy)]
pub struct Board {
    pub inner: [u8; 81],
}

impl Board {
    pub fn empty_squares(&self) -> usize {
        self.inner.iter().filter(|s| **s == 0).count()
    }

    // Returns true if the board is in a solved state.
    pub fn is_solved(&self) -> bool {
        for i in 0..self.inner.len() {
            let s = Square(i);
            let constraints = self.get_constraints_for_square(s);
            for i in 1_u8..10 {
                if constraints & 1 << i == 0 {
                    return false;
                }
            }
        }

        true
    }

    fn get_moves_for_square(&self, square: Square) -> u16 {
        !self.get_constraints_for_square(square)
    }
    fn get_constraints_for_square(&self, square: Square) -> u16 {
        let mut constraints = 0;

        for &v in self.get_row_for_square(square) {
            constraints |= 1 << v;
        }

        for v in self.get_col_for_square(square) {
            constraints |= 1 << v;
        }

        for v in self.get_block_for_square(square) {
            constraints |= 1 << v;
        }

        constraints
    }

    fn get_row_for_square(&self, square: Square) -> &[u8] {
        let start = square.row_start();
        &self.inner[start..start + 9]
    }

    fn get_col_for_square(&self, square: Square) -> impl Iterator<Item = u8> + '_ {
        let col = square.col();

        let mut result = [0; 9];

        for row in 0..9 {
            result[row] = self.inner[row * 9 + col];
        }

        result.into_iter()
    }

    // Returns an iterator that yields the squares in the 3x3 block that contains the square.
    fn get_block_for_square(&self, square: Square) -> impl Iterator<Item = u8> + '_ {
        let x = square.col() / 3;
        let y = square.row() / 3;

        self.get_block(x, y).map(|s| self.inner[s.0])
    }

    fn get_block(&self, x: usize, y: usize) -> impl Iterator<Item = Square> {
        assert!(x < 3);
        assert!(y < 3);
        let row = y * 3;
        let row_start = row * 9;
        let col = x * 3;

        let mut result = [Square(0); 9];

        for i in 0..3 {
            let row_start = row_start + i * 9 + col;
            for col in 0..3 {
                result[i * 3 + col] = Square(row_start + col);
            }
        }

        result.into_iter()
    }

    pub fn print(&self) {
        for i in 0..self.inner.len() / 3 {
            if i % 3 == 0 && i != 0 {
                println!();
            }
            if i % 9 == 0 {
                println!("+---------+---------+---------+");
            }

            if i % 3 == 0 {
                print!("|");
            }
            print!(
                " {}  {}  {} |",
                self.inner[i * 3],
                self.inner[i * 3 + 1],
                self.inner[i * 3 + 2]
            );
        }
        println!("\n+---------+---------+---------+");
    }
}

#[derive(Debug, Copy, Clone)]
/// IMPORTANT: value must be < 81.
struct Square(usize);

impl Square {
    fn row(&self) -> usize {
        let row = self.0 / 9;
        if row < 9 {
            row
        } else {
            // SAFETY: Square is always < 81, so this is unreachable.
            unsafe { unreachable_unchecked() }
        }
    }

    fn row_start(&self) -> usize {
        let row_start = self.row() * 9;
        if row_start <= 72 {
            row_start
        } else {
            // SAFETY: Square is always < 81, so this is unreachable.
            unsafe { unreachable_unchecked() }
        }
    }

    fn col(&self) -> usize {
        let col = self.0 - self.row_start();
        if col < 9 {
            col
        } else {
            // SAFETY: Square is always < 81, so this is unreachable.
            unsafe { unreachable_unchecked() }
        }
    }
}

#[derive(Clone)]
struct SpeculationState {
    pub board: Board,
    empty_squares: usize,
    // Stores the valid moves for each square on the board.
    valid_moves: [u16; 81],
    row_values: [u16; 9],
    col_values: [u16; 9],
    block_values: [u16; 9],
}

impl SpeculationState {
    fn new_initial(input: Board) -> Self {
        let mut initial = SpeculationState {
            empty_squares: input.empty_squares(),
            board: input,
            valid_moves: [0; 81],
            row_values: [0; 9],
            col_values: [0; 9],
            block_values: [0; 9],
        };

        // Initialize the valid moves for each square.
        for square in 0..input.inner.len() {
            if input.inner[square] == 0 {
                initial.valid_moves[square] = input.get_moves_for_square(Square(square));
            }
        }

        // Initialize the bookkeeping of values present in each row/column/block.

        for row in 0..9 {
            let row_start = row * 9;

            let mut present = 0_u16;

            for col in 0..9 {
                let square = row_start + col;
                present |= 1 << initial.board.inner[square];
            }
            initial.row_values[row] = present;
        }

        for col_start in 0..9 {
            let start_square = Square(col_start);

            let mut present = 0_u16;

            for v in initial.board.get_col_for_square(start_square) {
                present |= 1 << v;
            }

            initial.col_values[col_start] = present;
        }

        for x in 0..3 {
            for y in 0..3 {
                let block = initial.board.get_block(x, y);

                let mut present = 0_u16;

                for square in block {
                    present |= 1 << initial.board.inner[square.0];
                }

                initial.block_values[y * 3 + x] = present
            }
        }

        initial
    }

    fn is_solved(&self) -> bool {
        self.empty_squares == 0
    }

    // Updates the state to reflect a move.
    fn make_move(&mut self, square: Square, value: u8) {
        let idx = square.0;
        self.board.inner[idx] = value;
        self.empty_squares -= 1;

        // Update valid moves for affected squares.

        // Not more moves are valid for this square.
        self.valid_moves[idx] = 0;

        let row = square.row();

        self.row_values[row] |= 1 << value;

        let row_start = square.row_start();

        // Row
        for square in row_start..row_start + 9 {
            self.valid_moves[square] &= !(1 << value);
        }

        let col = square.col();

        self.col_values[col] |= 1 << value;

        // Col
        for row in 0..9 {
            let row_start = row * 9;

            let square = row_start + col;

            self.valid_moves[square] &= !(1 << value);
        }

        // Block
        {
            let x = col / 3;
            let y = row / 3;

            self.block_values[y * 3 + x] |= 1 << value;

            for square in self.board.get_block(x, y) {
                self.valid_moves[square.0] &= !(1 << value);
            }
        }
    }
}

pub fn solve(input: Board, print_dbg: bool) -> Board {
    let mut initial = SpeculationState::new_initial(input);

    Solver::place_obvious(&mut initial);

    if initial.is_solved() {
        return initial.board;
    }

    let mut initial_moves = Vec::new();

    for (square, moves) in initial.valid_moves.iter().enumerate() {
        if *moves != 0 {
            for i in 1_u8..10 {
                if moves & 1 << i != 0 {
                    initial_moves.push((Square(square), i));
                }
            }
        }
    }

    if print_dbg {
        println!(
            "Number of initial states to explore: {}",
            initial_moves.len()
        );
    }

    let solved = AtomicBool::new(false);

    let result = initial_moves.into_par_iter().find_map_any(|initial_move| {
        if solved.load(Ordering::Relaxed) {
            return None;
        }
        let mut initial_state = initial.clone();
        initial_state.make_move(initial_move.0, initial_move.1);
        let result = Solver::default().solve(initial_state, &solved);
        if result.is_some() {
            if print_dbg {
                println!("Found solution");
            }
            solved.store(true, Ordering::Relaxed);
        }
        result
    });

    match result {
        Some(res) => res,
        None => initial.board,
    }
}

#[derive(Default)]
pub struct Solver {
    speculation_stack: Vec<SpeculationState>,
}

impl Solver {
    fn solve(&mut self, mut initial: SpeculationState, solved: &AtomicBool) -> Option<Board> {
        Solver::place_obvious(&mut initial);

        if initial.is_solved() {
            return Some(initial.board);
        }

        self.speculation_stack.push(initial);

        loop {
            if solved.load(Ordering::Relaxed) {
                return None;
            }

            loop {
                if let Some(new) = self.get_speculation() {
                    self.speculation_stack.push(new);

                    break;
                } else {
                    self.speculation_stack.pop();
                    if self.speculation_stack.is_empty() {
                        return None;
                    }
                }
            }

            let state = self.speculation_stack.last_mut().unwrap();

            Solver::place_obvious(state);

            if state.is_solved() {
                return Some(state.board);
            }
        }
    }

    fn get_speculation(&mut self) -> Option<SpeculationState> {
        let prev = self.speculation_stack.last_mut().unwrap();

        for (square, moves) in prev.valid_moves.iter().enumerate() {
            if *moves != 0 {
                for i in 1_u8..10 {
                    if moves & 1 << i != 0 {
                        let mut state = prev.clone();
                        state.make_move(Square(square), i);
                        // Either we solve the puzzle on this speculation path,
                        // or this guess is wrong and is there for not a valid
                        // move to try again.
                        prev.valid_moves[square] &= !(1 << i);
                        return Some(state);
                    }
                }
            }
        }

        None
    }

    fn place_obvious(state: &mut SpeculationState) {
        let mut made_move = true;

        while made_move {
            made_move = false;
            for square in 0..state.valid_moves.len() {
                let moves = state.valid_moves[square];
                if moves != 0 {
                    let mut possible_moves = 0_u8;
                    let mut possible_move = 0;

                    for i in 1_u8..10 {
                        if moves & 1 << i != 0 {
                            possible_moves += 1;

                            possible_move = i;
                        }
                    }

                    if possible_moves == 1 {
                        state.make_move(Square(square), possible_move);
                        made_move = true;
                    }
                }
            }

            // Try fill rows
            for row in 0..9 {
                let row_start = row * 9;

                let present = state.row_values[row];

                for i in 1..10_u8 {
                    if present & 1 << i == 0 {
                        // Missing

                        // The i-th bool represents if the missing number can be placed in
                        // the i-th square.
                        let mut can_place = [true; 9];
                        let mut count = 9;

                        for col in 0..9 {
                            let square = row_start + col;

                            let present = state.valid_moves[square] & 1 << i == 0;
                            can_place[col] = !present;
                            count -= present as u8;
                        }

                        if count == 1 {
                            let index = can_place.iter().position(|f| *f).unwrap();

                            state.make_move(Square(row_start + index), i);
                            made_move = true;
                        }
                    }
                }
            }

            // Try fill columns
            for col_start in 0..9 {
                let present = state.col_values[col_start];

                for i in 1..10_u8 {
                    if present & 1 << i == 0 {
                        // Missing

                        // The i-th bool represents if the missing number can be placed in
                        // the i-th square.
                        let mut can_place = [true; 9];
                        let mut count = 9;

                        for row in 0..9 {
                            let row_start = row * 9;
                            let square = row_start + col_start;

                            let present = state.valid_moves[square] & 1 << i == 0;
                            can_place[row] = !present;
                            count -= present as u8;
                        }

                        if count == 1 {
                            let index = can_place.iter().position(|f| *f).unwrap();

                            let row_start = index * 9;

                            state.make_move(Square(row_start + col_start), i);
                            made_move = true;
                        }
                    }
                }
            }

            // Try fill blocks
            for x in 0..3 {
                for y in 0..3 {
                    let present = state.block_values[y * 3 + x];

                    for i in 1..10_u8 {
                        if present & 1 << i == 0 {
                            // Missing

                            // The i-th bool represents if the missing number can be placed in
                            // the i-th square.
                            let mut can_place = [true; 9];
                            let mut count = 9;

                            let block = state.board.get_block(x, y);

                            for (block_idx, square) in block.enumerate() {
                                let present = state.valid_moves[square.0] & 1 << i == 0;
                                can_place[block_idx] = !present;
                                count -= present as u8;
                            }

                            if count == 1 {
                                let index = can_place.iter().position(|f| *f).unwrap();
                                let square = state.board.get_block(x, y).nth(index).unwrap();

                                state.make_move(Square(square.0), i);
                                made_move = true;
                            }
                        }
                    }
                }
            }
        }
    }
}
