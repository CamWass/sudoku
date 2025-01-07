use rayon::prelude::*;
use std::{
    hint::unreachable_unchecked,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
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

        // SAFETY: Squares are always in-bounds by construction.
        Board::get_block(x, y).map(|s| unsafe { *self.inner.get_unchecked(s.0) })
    }

    fn get_block(x: usize, y: usize) -> impl Iterator<Item = Square> {
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
                let block = Board::get_block(x, y);

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

            for square in Board::get_block(x, y) {
                self.valid_moves[square.0] &= !(1 << value);
            }
        }
    }
}

pub fn solve(input: Board, print_dbg: bool) -> (Board, bool) {
    let mut initial = SpeculationState::new_initial(input);

    Solver::place_obvious(&mut initial);

    if initial.is_solved() {
        return (initial.board, initial.board.is_solved());
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

    // TODO: better parallelism? https://github.com/judofyr/spice/issues/5
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
        Some(res) => (res, true),
        None => (initial.board, false),
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
                let prev = self.speculation_stack.last_mut().unwrap();
                if let Some(new) = Self::get_speculation(prev) {
                    self.speculation_stack.push(new);

                    break;
                }

                self.speculation_stack.pop();
                if self.speculation_stack.is_empty() {
                    return None;
                }
            }

            let state = self.speculation_stack.last_mut().unwrap();

            Solver::place_obvious(state);

            if state.is_solved() {
                return Some(state.board);
            }
        }
    }

    fn get_speculation(prev: &mut SpeculationState) -> Option<SpeculationState> {
        let res = prev
            .valid_moves
            .iter()
            .enumerate()
            .map(|(s, moves)| (s, moves.count_ones()))
            .filter(|(_, moves)| *moves > 6)
            .min_by_key(|(_, m)| *m);

        if let Some((square, _)) = res {
            let moves = prev.valid_moves[square];
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

        None
    }

    fn place_obvious(state: &mut SpeculationState) {
        let mut made_move = true;

        while made_move {
            made_move = false;

            Self::fill_squares(state, &mut made_move);
            Self::fill_rows(state, &mut made_move);
            Self::fill_columns(state, &mut made_move);
            Self::fill_blocks(state, &mut made_move);
        }
    }

    fn fill_squares(state: &mut SpeculationState, made_move: &mut bool) {
        // TODO: we can operate on batches of squares, potentially enabling vectorisation
        // and elimination of some shared computation (updating valid moves etc)
        for square in 0..state.valid_moves.len() {
            let moves = state.valid_moves[square];
            if moves != 0 {
                let mut possible_moves = 0_u8;
                let mut possible_move = 0;

                for i in 1_u8..10 {
                    let can_place = moves & 1 << i != 0;
                    possible_move |= i * can_place as u8;
                    possible_moves += can_place as u8;
                }

                if possible_moves == 1 {
                    state.make_move(Square(square), possible_move);
                    *made_move = true;
                }
            }
        }
    }

    fn fill_rows(state: &mut SpeculationState, made_move: &mut bool) {
        for row in 0..9 {
            let row_start = row * 9;

            let present = state.row_values[row];

            for i in 1..10_u8 {
                if present & 1 << i == 0 {
                    // Missing

                    let mut index = 0;
                    let mut count = 0;

                    for col in 0..9 {
                        let square = row_start + col;

                        let can_place = state.valid_moves[square] & 1 << i != 0;
                        index |= col * can_place as usize;
                        count += can_place as usize;
                    }

                    if count == 1 {
                        state.make_move(Square(row_start + index), i);
                        *made_move = true;
                    }
                }
            }
        }
    }

    fn fill_columns(state: &mut SpeculationState, made_move: &mut bool) {
        for col_start in 0..9 {
            let present = state.col_values[col_start];

            for i in 1..10_u8 {
                if present & 1 << i == 0 {
                    // Missing

                    let mut index = 0;
                    let mut count = 0;

                    for row in 0..9 {
                        let row_start = row * 9;
                        let square = row_start + col_start;

                        let can_place = state.valid_moves[square] & 1 << i != 0;
                        index |= row * can_place as usize;
                        count += can_place as usize;
                    }

                    if count == 1 {
                        let row_start = index * 9;

                        state.make_move(Square(row_start + col_start), i);
                        *made_move = true;
                    }
                }
            }
        }
    }

    fn fill_blocks(state: &mut SpeculationState, made_move: &mut bool) {
        for x in 0..3 {
            for y in 0..3 {
                let present = state.block_values[y * 3 + x];

                for i in 1..10_u8 {
                    if present & 1 << i == 0 {
                        // Missing

                        let mut index = 0;
                        let mut count = 0;

                        let block = Board::get_block(x, y);

                        for (block_idx, square) in block.enumerate() {
                            let can_place = state.valid_moves[square.0] & 1 << i != 0;
                            index |= block_idx * can_place as usize;
                            count += can_place as usize;
                        }

                        if count == 1 {
                            let square = Board::get_block(x, y).nth(index).unwrap();

                            state.make_move(Square(square.0), i);
                            *made_move = true;
                        }
                    }
                }
            }
        }
    }
}

pub fn generate_solved_board() -> [u8; 81] {
    let mut squares = [0; 81];

    // Idea from https://gamedev.stackexchange.com/a/138228

    squares[..9].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    fastrand::shuffle(&mut squares[..9]);

    let mut i = 1;

    for row in 1..9 {
        let row_start = row * 9;
        let prev_row_start = (row - 1) * 9;
        let mut prev = [0; 9];
        prev.copy_from_slice(&squares[prev_row_start..prev_row_start + 9]);
        squares[row_start..row_start + 9].copy_from_slice(&prev);
        let shift = match i {
            0 => 1,
            1 => 3,
            2 => 3,
            _ => unreachable!(),
        };
        squares[row_start..row_start + 9].rotate_left(shift);

        i += 1;
        if i == 3 {
            i = 0;
        }
    }

    let mut indices = [0, 1, 2];

    // Shuffle rows in each block.
    for block in 0..3 {
        fastrand::shuffle(&mut indices);

        let mut buffer = [0; 27];

        for row in 0..3 {
            let src_row_start = block * 27 + row * 9;
            let dest_row = indices[row];
            let dest_row_start = dest_row * 9;
            buffer[dest_row_start..dest_row_start + 9]
                .copy_from_slice(&squares[src_row_start..src_row_start + 9]);
        }

        let block_start = block * 27;

        squares[block_start..block_start + 27].copy_from_slice(&buffer);
    }

    // Shuffle columns in each block.
    for block in 0..3 {
        fastrand::shuffle(&mut indices);

        let mut buffer = [0; 27];

        for col in 0..3 {
            let src_col_start = block * 3 + col;
            let dest_col = indices[col];

            for row in 0..9 {
                buffer[dest_col * 9 + row] = squares[row * 9 + src_col_start];
            }
        }

        for dest_col in 0..3 {
            let dest_col_start = block * 3 + dest_col;

            for row in 0..9 {
                squares[row * 9 + dest_col_start] = buffer[dest_col * 9 + row];
            }
        }
    }

    debug_assert!(Board { inner: squares }.is_solved());

    squares
}

fn count_solutions_from_state(mut initial: SpeculationState, solutions: &AtomicU32) {
    let mut speculation_stack = Vec::new();
    Solver::place_obvious(&mut initial);

    if initial.is_solved() {
        solutions.fetch_add(1, Ordering::Relaxed);
        return;
    }

    speculation_stack.push(initial);

    loop {
        loop {
            let prev = speculation_stack.last_mut().unwrap();
            if let Some(new) = Solver::get_speculation(prev) {
                speculation_stack.push(new);

                break;
            }

            speculation_stack.pop();
            if speculation_stack.is_empty() {
                return;
            }
        }

        let state = speculation_stack.last_mut().unwrap();

        Solver::place_obvious(state);

        if state.is_solved() {
            solutions.fetch_add(1, Ordering::Relaxed);
            return;
        }
    }
}

pub fn count_solutions(input: Board) -> u32 {
    let mut initial = SpeculationState::new_initial(input);

    Solver::place_obvious(&mut initial);

    if initial.is_solved() {
        return 1;
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

    let solutions = AtomicU32::new(0);

    initial_moves.into_par_iter().for_each(|initial_move| {
        let mut initial_state = initial.clone();
        initial_state.make_move(initial_move.0, initial_move.1);
        count_solutions_from_state(initial_state, &solutions);
    });

    solutions.into_inner()
}
