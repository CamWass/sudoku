// From https://en.wikipedia.org/wiki/Sudoku
#[rustfmt::skip] 
static INPUT: [u8; 81] = [
    5, 3, 0,  0, 7, 0,  0, 0, 0,
    6, 0, 0,  1, 9, 5,  0, 0, 0,
    0, 9, 8,  0, 0, 0,  0, 6, 0,

    8, 0, 0,  0, 6, 0,  0, 0, 3,
    4, 0, 0,  8, 0, 3,  0, 0, 1,
    7, 0, 0,  0, 2, 0,  0, 0, 6,

    0, 6, 0,  0, 0, 0,  2, 8, 0,
    0, 0, 0,  4, 1, 9,  0, 0, 5,
    0, 0, 0,  0, 8, 0,  0, 7, 9,
];
#[rustfmt::skip] 
static OUTPUT: [u8; 81]  = [
    5, 3, 4,  6, 7, 8,  9, 1, 2,
    6, 7, 2,  1, 9, 5,  3, 4, 8,
    1, 9, 8,  3, 4, 2,  5, 6, 7,

    8, 5, 9,  7, 6, 1,  4, 2, 3,
    4, 2, 6,  8, 5, 3,  7, 9, 1,
    7, 1, 3,  9, 2, 4,  8, 5, 6,
    
    9, 6, 1,  5, 3, 7,  2, 8, 4,
    2, 8, 7,  4, 1, 9,  6, 3, 5,
    3, 4, 5,  2, 8, 6,  1, 7, 9,
];
// static INPUT: [[u8; 9]; 9] = [
//     [5, 3, 0, 0, 7, 0, 0, 0, 0],
//     [6, 0, 0, 1, 9, 5, 0, 0, 0],
//     [0, 9, 8, 0, 0, 0, 0, 6, 0],
//     [8, 0, 0, 0, 6, 0, 0, 0, 3],
//     [4, 0, 0, 8, 0, 3, 0, 0, 1],
//     [7, 0, 0, 0, 2, 0, 0, 0, 6],
//     [0, 6, 0, 0, 0, 0, 2, 8, 0],
//     [0, 0, 0, 4, 1, 9, 0, 0, 5],
//     [0, 0, 0, 0, 8, 0, 0, 7, 9],
// ];
// static OUTPUT: [[u8; 9]; 9] = [
//     [5, 3, 4, 6, 7, 8, 9, 1, 2],
//     [6, 7, 2, 1, 9, 5, 3, 4, 8],
//     [1, 9, 8, 3, 4, 2, 5, 6, 7],
//     [8, 5, 9, 7, 6, 1, 4, 2, 3],
//     [4, 2, 6, 8, 5, 3, 7, 9, 1],
//     [7, 1, 3, 9, 2, 4, 8, 5, 6],
//     [9, 6, 1, 5, 3, 7, 2, 8, 4],
//     [2, 8, 7, 4, 1, 9, 6, 3, 5],
//     [3, 4, 5, 2, 8, 6, 1, 7, 9],
// ];

/*
Pick square
choose number not in row/column/box
continue until contradiction/ no options
backtrack
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

        self.get_block(x, y)
    }

    fn get_block(&self, x: usize, y: usize) -> impl Iterator<Item = u8> + '_ {
        let row = y * 3;
        let row_start = row * 9;
        let col = x * 3;
        let cur = row_start + col;
        let end = cur + 9 + 9 + 2;
        BlockIter {
            board: self,

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

struct BlockIter<'b> {
    board: &'b Board,

    cur: usize,
    end: usize,

    col_in_block: usize,
}

impl Iterator for BlockIter<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur > self.end {
            return None;
        }

        let square = self.board.inner[self.cur];

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

fn main() {
    let mut board = Board { inner: INPUT };

    println!("Input:");
    board.print();

    // for (s, _) in board.inner.iter().enumerate() {
    //     let s = Square(s);
    //     let col = board.get_block_for_square(s);
    //     for (i, s) in col.enumerate() {
    //         if i % 3 == 0 && i != 0 {
    //             print!("\n");
    //         }
    //         print!("{}, ", s)
    //     }
    //     println!("\n");
    //     // println!("{:?}", col.collect::<Vec<_>>());
    // }

    let mut possible_moves = vec![];

    let mut made_move = true;

    while made_move {
        made_move = false;
        for square in 0..board.inner.len() {
            if board.inner[square] != 0 {
                continue;
            }

            let square = Square(square);

            possible_moves.clear();

            let moves = board.get_moves_for_square(square);
            for i in 1_u8..10 {
                if moves & 1 << i != 0 {
                    possible_moves.push(i);
                }
            }

            if possible_moves.len() == 1 {
                board.inner[square.0] = possible_moves[0];
                made_move = true;
            }
        }
    }

    println!("Output:");
    board.print();

    if board.is_solved() {
        println!("Solved puzzle");
    } else {
        let empty_squares = board.inner.iter().filter(|s| **s == 0).count();

        println!(
            "Could not solve puzzle. {} empty squares remaining",
            empty_squares
        );
    }

    debug_assert!(board.inner == OUTPUT);
}
