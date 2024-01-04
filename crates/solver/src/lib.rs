use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

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
        let idx = square.0;
        let start = idx / 9 * 9;
        &self.inner[start..start + 9]
    }

    fn get_col_for_square(&self, square: Square) -> impl Iterator<Item = u8> + '_ {
        let idx = square.0;
        let row_start = idx / 9 * 9;
        let col = idx - row_start;
        ColIter {
            board: self,
            col,
            row: 0,
        }
    }

    // Returns an iterator that yields the squares in the 3x3 block that contains the square.
    fn get_block_for_square(&self, square: Square) -> impl Iterator<Item = u8> + '_ {
        let idx = square.0;
        let row_start = idx / 9 * 9;
        let col = idx - row_start;

        let x = col / 3;

        let row = idx / 9;

        let y = row / 3;

        self.get_block(x, y).map(|s| self.inner[s.0])
    }

    fn get_block(&self, x: usize, y: usize) -> impl Iterator<Item = Square> {
        let row = y * 3;
        let row_start = row * 9;
        let col = x * 3;
        let cur = row_start + col;
        let end = cur + 9 + 9 + 2;
        BlockIter {
            cur,
            end,
            col_in_block: 0,
        }
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
struct Square(usize);

struct ColIter<'b> {
    board: &'b Board,
    col: usize,
    row: usize,
}

impl Iterator for ColIter<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row == 9 {
            return None;
        }

        let row_start = self.row * 9;

        let square = self.board.inner[row_start + self.col];

        self.row += 1;

        Some(square)
    }
}

struct BlockIter {
    cur: usize,
    end: usize,

    col_in_block: usize,
}

impl Iterator for BlockIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur > self.end {
            return None;
        }

        let square = Square(self.cur);

        if self.col_in_block == 2 {
            self.cur += 9; // next row
            self.cur -= 2; // back to first col in block
            self.col_in_block = 0;
        } else {
            self.cur += 1;
            self.col_in_block += 1;
        }

        Some(square)
    }
}

struct SpeculationState {
    pub board: Board,
    empty_squares: usize,
    // Stores the valid moves for each square on the board.
    valid_moves: [u16; 81],
}

impl SpeculationState {
    fn is_solved(&self) -> bool {
        self.empty_squares == 0
    }

    // Updates the state to reflect a move.
    fn make_move(&mut self, square: usize, value: u8) {
        self.board.inner[square] = value;
        self.empty_squares -= 1;

        // Update valid moves for affected squares.

        // Not more moves are valid for this square.
        self.valid_moves[square] = 0;

        let row_start = square / 9 * 9;

        // Row
        for square in row_start..row_start + 9 {
            self.valid_moves[square] &= !(1 << value);
        }

        let col = square - row_start;

        // Col
        for row in 0..9 {
            let row_start = row * 9;

            let square = row_start + col;

            self.valid_moves[square] &= !(1 << value);
        }

        // Block
        {
            let x = col / 3;

            let row = square / 9;

            let y = row / 3;

            for square in self.board.get_block(x, y) {
                self.valid_moves[square.0] &= !(1 << value);
            }
        }
    }
}

pub fn solve(input: Board, print_dbg: bool) -> Board {
    let mut initial = SpeculationState {
        empty_squares: input.empty_squares(),
        board: input,
        valid_moves: [0; 81],
    };

    // Initialize the valid moves for each square.
    for square in 0..input.inner.len() {
        if input.inner[square] == 0 {
            initial.valid_moves[square] = input.get_moves_for_square(Square(square));
        }
    }

    Solver::place_obvious(&mut initial);

    if initial.is_solved() {
        return initial.board;
    }

    let mut initial_moves = Vec::new();

    for (square, moves) in initial.valid_moves.iter().enumerate() {
        if *moves != 0 {
            for i in 1_u8..10 {
                if moves & 1 << i != 0 {
                    initial_moves.push((square, i));
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
        let mut initial_state = SpeculationState {
            board: initial.board,
            empty_squares: initial.empty_squares,
            valid_moves: initial.valid_moves,
        };
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
                        let mut state = SpeculationState {
                            empty_squares: prev.empty_squares,
                            board: prev.board,
                            valid_moves: prev.valid_moves,
                        };
                        state.make_move(square, i);
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
                    let mut possible_move = None;

                    for i in 1_u8..10 {
                        if moves & 1 << i != 0 {
                            possible_moves += 1;

                            possible_move = Some(i);
                        }
                    }

                    if possible_moves == 1 {
                        state.make_move(square, possible_move.unwrap());
                        made_move = true;
                    }
                }
            }

            // Try fill rows
            for row in 0..9_u8 {
                let row_start = row * 9;

                let mut present = 0_u16;

                for col in 0..9_u8 {
                    let square = row_start + col;

                    present |= 1 << state.board.inner[square as usize];
                }

                for i in 1..10_u8 {
                    if present & 1 << i == 0 {
                        // Missing

                        // The first 9 bits are flags for whether the missing number
                        // can be placed in that i-th square.
                        let mut can_place = [true; 9];

                        for col in 0..9_u8 {
                            let square = (row_start + col) as usize;

                            if state.valid_moves[square] & 1 << i == 0 {
                                can_place[col as usize] = false;
                            }
                        }

                        // We want one bit set, but the last 7 bits will always be one.
                        // So there will always be 7 ones.
                        if can_place.iter().filter(|f| **f).count() == 1 {
                            let index = can_place.iter().position(|f| *f).unwrap();

                            state.make_move(row_start as usize + index as usize, i);
                            made_move = true;
                        }
                    }
                }
            }

            // Try fill columns
            for col_start in 0..9_u8 {
                let start_square = Square(col_start as usize);
                let col = state.board.get_col_for_square(start_square);

                let mut present = 0_u16;

                for v in col {
                    present |= 1 << v;
                }

                for i in 1..10_u8 {
                    if present & 1 << i == 0 {
                        // Missing

                        // The first 9 bits are flags for whether the missing number
                        // can be placed in that i-th square.
                        let mut can_place = [true; 9];

                        for row in 0..9_u8 {
                            let row_start = row * 9;
                            let square = (row_start + col_start) as usize;

                            if state.valid_moves[square] & 1 << i == 0 {
                                can_place[row as usize] = false;
                            }
                        }

                        // We want one bit set, but the last 7 bits will always be one.
                        // So there will always be 7 ones.
                        if can_place.iter().filter(|f| **f).count() == 1 {
                            let index = can_place.iter().position(|f| *f).unwrap();

                            let row_start = index * 9;

                            state.make_move(row_start as usize + col_start as usize, i);
                            made_move = true;
                        }
                    }
                }
            }

            // Try fill blocks
            for x in 0..3 {
                for y in 0..3 {
                    let block = state.board.get_block(x, y);

                    let mut present = 0_u16;

                    for square in block {
                        present |= 1 << state.board.inner[square.0];
                    }

                    for i in 1..10_u8 {
                        if present & 1 << i == 0 {
                            // Missing

                            // The first 9 bits are flags for whether the missing number
                            // can be placed in that i-th square.
                            let mut can_place = [true; 9];

                            let block = state.board.get_block(x, y);

                            for (block_idx, square) in block.enumerate() {
                                if state.valid_moves[square.0] & 1 << i == 0 {
                                    can_place[block_idx] = false;
                                }
                            }

                            // We want one bit set, but the last 7 bits will always be one.
                            // So there will always be 7 ones.
                            if can_place.iter().filter(|f| **f).count() == 1 {
                                let index = can_place.iter().position(|f| *f).unwrap();
                                let square =
                                    state.board.get_block(x, y).nth(index as usize).unwrap();

                                state.make_move(square.0, i);
                                made_move = true;
                            }
                        }
                    }
                }
            }
        }
    }
}
