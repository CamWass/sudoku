use solver::{solve as solve_inner, Board};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn solve(input: &[u8], output: &mut [u8]) {
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

    output.copy_from_slice(&board.inner);
}
