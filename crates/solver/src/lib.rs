use std::hint::unreachable_unchecked;

pub mod graph;
pub mod intuitive;

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
        self.get_block(x, y)
            .map(|s| unsafe { *self.inner.get_unchecked(s.0) })
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
