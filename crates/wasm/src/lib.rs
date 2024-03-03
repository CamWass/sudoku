use solver::{solve as solve_inner, Board};
use wasm_bindgen::prelude::*;

pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub fn generate_solved_board(output: &mut [u8]) {
    output.copy_from_slice(&solver::generate_solved_board());
}

#[wasm_bindgen]
pub fn solve(input: &[u8], output: &mut [u8]) -> bool {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let input = Board {
        inner: input.try_into().unwrap(),
    };

    input.print();

    let board = solve_inner(input, false);

    output.copy_from_slice(&board.0.inner);
    board.1
}
