mod puzzles;

use crate::puzzles::*;

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
*/

struct Board {
    inner: [u8; 81],
}

impl Board {
    // Returns true if the board is in  a solved state.
    fn is_solved(&self) -> bool {
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

    fn print(&self) {
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

struct Solver {
    board: Board,
}

impl Solver {
    fn solve(&mut self) {
        let mut possible_moves = vec![];

        let mut made_move = true;

        while made_move {
            made_move = false;
            for square in 0..self.board.inner.len() {
                if self.board.inner[square] != 0 {
                    continue;
                }

                let square = Square(square);

                possible_moves.clear();

                let moves = self.board.get_moves_for_square(square);
                for i in 1_u8..10 {
                    if moves & 1 << i != 0 {
                        possible_moves.push(i);
                    }
                }

                if possible_moves.len() == 1 {
                    self.board.inner[square.0] = possible_moves[0];
                    made_move = true;
                }
            }

            // Try fill rows
            for row in 0..9_u8 {
                let row_start = row * 9;

                let mut present = 0_u16;

                for col in 0..9_u8 {
                    let square = row_start + col;

                    present |= 1 << self.board.inner[square as usize];
                }

                for i in 1..10_u8 {
                    if present & 1 << i == 0 {
                        // Missing

                        // The first 9 bits are flags for whether the missing number
                        // can be placed in that i-th square.
                        let mut can_place = u16::MAX;

                        for col in 0..9_u8 {
                            let square = (row_start + col) as usize;

                            if self.board.inner[square] != 0 {
                                can_place &= !(1 << col);
                                continue;
                            }

                            let square = Square(square);

                            for v in self.board.get_col_for_square(square) {
                                if v == i {
                                    can_place &= !(1 << col);
                                }
                            }

                            for v in self.board.get_block_for_square(square) {
                                if v == i {
                                    can_place &= !(1 << col);
                                }
                            }
                        }

                        // We want one bit set, but the last 7 bits will always be one.
                        // So there will always be 7 ones.
                        if can_place.count_ones() == 1 + 7 {
                            let index = can_place.trailing_zeros();

                            self.board.inner[(row_start as usize + index as usize)] = i;
                            made_move = true;
                        }
                    }
                }
            }

            // Try fill columns
            for col_start in 0..9_u8 {
                let start_square = Square(col_start as usize);
                let col = self.board.get_col_for_square(start_square);

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

                            if self.board.inner[square] != 0 {
                                can_place &= !(1 << row);
                                continue;
                            }

                            let square = Square(square);

                            for &v in self.board.get_row_for_square(square) {
                                if v == i {
                                    can_place &= !(1 << row);
                                }
                            }

                            for v in self.board.get_block_for_square(square) {
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

                            self.board.inner[(row_start as usize + col_start as usize)] = i;
                            made_move = true;
                        }
                    }
                }
            }

            // Try fill blocks
            for x in 0..3 {
                for y in 0..3 {
                    let block = self.board.get_block(x, y);

                    let mut present = 0_u16;

                    for square in block {
                        present |= 1 << self.board.inner[square.0];
                    }

                    for i in 1..10_u8 {
                        if present & 1 << i == 0 {
                            // Missing

                            // The first 9 bits are flags for whether the missing number
                            // can be placed in that i-th square.
                            let mut can_place = u16::MAX;

                            let block = self.board.get_block(x, y);

                            for (block_idx, square) in block.enumerate() {
                                if self.board.inner[square.0] != 0 {
                                    can_place &= !(1 << block_idx);
                                    continue;
                                }

                                for &v in self.board.get_row_for_square(square) {
                                    if v == i {
                                        can_place &= !(1 << block_idx);
                                    }
                                }

                                for v in self.board.get_col_for_square(square) {
                                    if v == i {
                                        can_place &= !(1 << block_idx);
                                    }
                                }
                            }

                            // We want one bit set, but the last 7 bits will always be one.
                            // So there will always be 7 ones.
                            if can_place.count_ones() == 1 + 7 {
                                let index = can_place.trailing_zeros();
                                let square =
                                    self.board.get_block(x, y).nth(index as usize).unwrap();

                                self.board.inner[square.0] = i;
                                made_move = true;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let input = &INPUT3;
    let mut solver = Solver {
        board: Board { inner: *input },
    };

    println!("Input:");
    solver.board.print();

    solver.solve();

    println!("Output:");
    solver.board.print();

    if solver.board.is_solved() {
        println!("Solved puzzle");
    } else {
        let old_empty_squares = input.iter().filter(|s| **s == 0).count();
        let new_empty_squares = solver.board.inner.iter().filter(|s| **s == 0).count();

        let placed = old_empty_squares - new_empty_squares;

        println!(
            "Could not solve puzzle. {} empty squares remaining (placed {})",
            new_empty_squares, placed
        );
    }

    debug_assert!(solver.board.inner == OUTPUT3);
}
