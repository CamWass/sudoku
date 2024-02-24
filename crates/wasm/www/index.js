import init, { solve } from "./wasm/wasm.js";

await init();

/* prettier-ignore */
const initial = [
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

const puzzleInput = document.getElementById("puzzle");

// Create all 81 squares and pre-populate them with the initial puzzle.
for (let i = 0; i < 81; i++) {
  const square = document.createElement("input");
  square.classList.add("square");
  square.value = initial[i] == 0 ? "" : initial[i];
  puzzleInput.appendChild(square);
}

// Clear the puzzle input when the 'clear' button is clicked.
document.getElementById("clear").addEventListener("click", () => {
  puzzleInput.childNodes.forEach((node) => {
    node.value = "";
  });
});

// When the 'Solve' button is clicked, collect the puzzle from the inputs and
// submit it to the solver. Then update the inputs to reflect the output.
document.getElementById("solve").addEventListener("click", () => {
  // Collect input.
  const input = new Uint8Array(81);
  puzzleInput.childNodes.forEach((node, i) => {
    input[i] = node.value;
  });

  // Submit to solver.
  const output = new Uint8Array(81);
  solve(input, output);

  // Update inputs.
  puzzleInput.childNodes.forEach((node, i) => {
    node.value = output[i] == 0 ? "" : output[i];
  });
});
