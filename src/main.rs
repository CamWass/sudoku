// From https://en.wikipedia.org/wiki/Sudoku
#[rustfmt::skip] 
static INPUT: [u8; 81] = [
    5, 3, 0, 0, 7, 0, 0, 0, 0,
    6, 0, 0, 1, 9, 5, 0, 0, 0,
    0, 9, 8, 0, 0, 0, 0, 6, 0,
    8, 0, 0, 0, 6, 0, 0, 0, 3,
    4, 0, 0, 8, 0, 3, 0, 0, 1,
    7, 0, 0, 0, 2, 0, 0, 0, 6,
    0, 6, 0, 0, 0, 0, 2, 8, 0,
    0, 0, 0, 4, 1, 9, 0, 0, 5,
    0, 0, 0, 0, 8, 0, 0, 7, 9,
];
#[rustfmt::skip] 
static OUTPUT: [u8; 81]  = [
    5, 3, 4, 6, 7, 8, 9, 1, 2,
    6, 7, 2, 1, 9, 5, 3, 4, 8,
    1, 9, 8, 3, 4, 2, 5, 6, 7,
    8, 5, 9, 7, 6, 1, 4, 2, 3,
    4, 2, 6, 8, 5, 3, 7, 9, 1,
    7, 1, 3, 9, 2, 4, 8, 5, 6,
    9, 6, 1, 5, 3, 7, 2, 8, 4,
    2, 8, 7, 4, 1, 9, 6, 3, 5,
    3, 4, 5, 2, 8, 6, 1, 7, 9,
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
    // Assumes the board is valid.
    fn is_solved(&self) -> bool {
        !self.inner.contains(&0)
    }

    fn get_empty_square(&self) -> Option<Square> {
        for (index, value) in self.inner.iter().enumerate() {
            if *value == 0 {
                return Some(Square(index));
            }
        }

        None
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
        // let idx = square.0;
        // let row_start = idx / 9 * 9;
        // let col = idx - row_start;
        // NeighbourIter {
        //     board: self,
        //     col,
        //     row: 0,
        // }
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
        if self.row >= 9 {
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
    cur_col: usize,
    cur_row: usize,
    // Inclusive
    start_col: usize,
    // Exclusive
    end_row: usize,
    end_col: usize,
}

impl Iterator for BlockIter<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: end conditions

        let square = self.board.inner[]
    }
}

fn main() {
    let mut board = Board { inner: INPUT };

    for (s, _) in board.inner.iter().enumerate() {
        let s = Square(s);
        let col = board.get_col_for_square(s);
        println!("{:?}", col.collect::<Vec<_>>());
    }

    // while let Some(empty) = board.get_empty_square() {
    //     dbg!(
    //         empty,
    //         &board.inner[empty.0],
    //         board.get_row_for_square(empty)
    //     );
    //     panic!();
    // }
}
