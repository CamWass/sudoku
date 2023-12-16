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

    // Returns true if the board is in  a solved state.
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
    // For each empty square in the board, stores the moves that we have already
    // tried in this speculation path.
    tried_moves: [u16; 81],
    // TODO: u8
    empty_squares: usize,
}

#[derive(Default)]
pub struct Solver {
    speculation_stack: Vec<SpeculationState>,
}

impl Solver {
    pub fn solve(&mut self, input: Board) -> Board {
        self.speculation_stack.push(SpeculationState {
            empty_squares: input.empty_squares(),
            board: input,
            tried_moves: [0; 81],
        });

        self.place_obvious();

        if self.is_solved() {
            return self.speculation_stack.pop().unwrap().board;
        }

        loop {
            loop {
                if let Some(new) = self.get_speculation() {
                    self.speculation_stack.push(new);

                    break;
                } else {
                    debug_assert!(self.speculation_stack.len() > 1);
                    self.speculation_stack.pop();
                }
            }

            self.place_obvious();

            if self.is_solved() {
                return self.speculation_stack.pop().unwrap().board;
            }
        }
    }

    fn get_speculation(&mut self) -> Option<SpeculationState> {
        let prev = self.speculation_stack.last_mut().unwrap();

        for square in 0..prev.board.inner.len() {
            if prev.board.inner[square] != 0 {
                continue;
            }

            let square = Square(square);

            let moves = prev.board.get_moves_for_square(square) & !prev.tried_moves[square.0];
            for i in 1_u8..10 {
                if moves & 1 << i != 0 {
                    let mut board = prev.board;
                    board.inner[square.0] = i;
                    prev.tried_moves[square.0] |= 1 << i;
                    return Some(SpeculationState {
                        empty_squares: prev.empty_squares - 1,
                        board,
                        tried_moves: prev.tried_moves,
                    });
                }
            }
        }

        None
    }

    fn is_solved(&self) -> bool {
        self.speculation_stack.last().unwrap().empty_squares == 0
    }

    fn place_obvious(&mut self) {
        let speculation_state = self.speculation_stack.last_mut().unwrap();
        let board = &mut speculation_state.board;

        let mut made_move = true;

        while made_move {
            made_move = false;
            for square in 0..board.inner.len() {
                if board.inner[square] != 0 {
                    continue;
                }

                let square = Square(square);

                let mut possible_moves = 0_u8;
                let mut possible_move = None;

                let moves = board.get_moves_for_square(square);
                for i in 1_u8..10 {
                    if moves & 1 << i != 0 {
                        possible_moves += 1;

                        possible_move = Some(i);
                    }
                }

                if possible_moves == 1 {
                    board.inner[square.0] = possible_move.unwrap();
                    made_move = true;
                    speculation_state.empty_squares -= 1;
                }
            }

            // Try fill rows
            for row in 0..9_u8 {
                let row_start = row * 9;

                let mut present = 0_u16;

                for col in 0..9_u8 {
                    let square = row_start + col;

                    present |= 1 << board.inner[square as usize];
                }

                for i in 1..10_u8 {
                    if present & 1 << i == 0 {
                        // Missing

                        // The first 9 bits are flags for whether the missing number
                        // can be placed in that i-th square.
                        let mut can_place = u16::MAX;

                        for col in 0..9_u8 {
                            let square = (row_start + col) as usize;

                            if board.inner[square] != 0 {
                                can_place &= !(1 << col);
                                continue;
                            }

                            let square = Square(square);

                            for v in board.get_col_for_square(square) {
                                if v == i {
                                    can_place &= !(1 << col);
                                }
                            }

                            for v in board.get_block_for_square(square) {
                                if v == i {
                                    can_place &= !(1 << col);
                                }
                            }
                        }

                        // We want one bit set, but the last 7 bits will always be one.
                        // So there will always be 7 ones.
                        if can_place.count_ones() == 1 + 7 {
                            let index = can_place.trailing_zeros();

                            board.inner[(row_start as usize + index as usize)] = i;
                            made_move = true;
                            speculation_state.empty_squares -= 1;
                        }
                    }
                }
            }

            // Try fill columns
            for col_start in 0..9_u8 {
                let start_square = Square(col_start as usize);
                let col = board.get_col_for_square(start_square);

                let mut present = 0_u16;

                for v in col {
                    present |= 1 << v;
                }

                for i in 1..10_u8 {
                    if present & 1 << i == 0 {
                        // Missing

                        // The first 9 bits are flags for whether the missing number
                        // can be placed in that i-th square.
                        let mut can_place = u16::MAX;

                        for row in 0..9_u8 {
                            let row_start = row * 9;
                            let square = (row_start + col_start) as usize;

                            if board.inner[square] != 0 {
                                can_place &= !(1 << row);
                                continue;
                            }

                            let square = Square(square);

                            for &v in board.get_row_for_square(square) {
                                if v == i {
                                    can_place &= !(1 << row);
                                }
                            }

                            for v in board.get_block_for_square(square) {
                                if v == i {
                                    can_place &= !(1 << row);
                                }
                            }
                        }

                        // We want one bit set, but the last 7 bits will always be one.
                        // So there will always be 7 ones.
                        if can_place.count_ones() == 1 + 7 {
                            let index = can_place.trailing_zeros();

                            let row_start = index * 9;

                            board.inner[(row_start as usize + col_start as usize)] = i;
                            made_move = true;
                            speculation_state.empty_squares -= 1;
                        }
                    }
                }
            }

            // Try fill blocks
            for x in 0..3 {
                for y in 0..3 {
                    let block = board.get_block(x, y);

                    let mut present = 0_u16;

                    for square in block {
                        present |= 1 << board.inner[square.0];
                    }

                    for i in 1..10_u8 {
                        if present & 1 << i == 0 {
                            // Missing

                            // The first 9 bits are flags for whether the missing number
                            // can be placed in that i-th square.
                            let mut can_place = u16::MAX;

                            let block = board.get_block(x, y);

                            for (block_idx, square) in block.enumerate() {
                                if board.inner[square.0] != 0 {
                                    can_place &= !(1 << block_idx);
                                    continue;
                                }

                                for &v in board.get_row_for_square(square) {
                                    if v == i {
                                        can_place &= !(1 << block_idx);
                                    }
                                }

                                for v in board.get_col_for_square(square) {
                                    if v == i {
                                        can_place &= !(1 << block_idx);
                                    }
                                }
                            }

                            // We want one bit set, but the last 7 bits will always be one.
                            // So there will always be 7 ones.
                            if can_place.count_ones() == 1 + 7 {
                                let index = can_place.trailing_zeros();
                                let square = board.get_block(x, y).nth(index as usize).unwrap();

                                board.inner[square.0] = i;
                                made_move = true;
                                speculation_state.empty_squares -= 1;
                            }
                        }
                    }
                }
            }
        }
    }
}
