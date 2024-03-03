import init, {
  initThreadPool,
  solve,
  generate_solved_board,
} from "./wasm/wasm.js";

await init();

const threadCount = navigator.hardwareConcurrency;

await initThreadPool(threadCount);

document.getElementById("threadCount").textContent = threadCount;

/* prettier-ignore */
const initial = [
  0,0,0, 0,0,0, 0,0,0,
  0,0,0, 0,0,0, 0,0,0,
  0,0,0, 0,0,0, 6,0,0,

  0,0,0, 0,9,0, 0,0,0,
  0,0,0, 0,0,0, 0,0,0,
  0,0,0, 0,4,0, 0,6,0,

  0,0,0, 0,0,0, 0,0,0,
  0,0,0, 0,0,0, 0,0,0,
  0,0,0, 0,0,0, 0,0,0,
];

const colours = [
  "#ffadad",
  "#ffd6a5",
  "#fdffb6",
  "#caffbf",
  "#96e8ff",
  "#a0c4ff",
  "#bdb2ff",
  "#debcff",
  "#ffc6ff",
  "#ffa7dc",
];

const solvedMsg = document.getElementById("solved");

const puzzleInput = document.getElementById("puzzle");

// Create all 81 squares and pre-populate them with the initial puzzle.
for (let i = 0; i < 81; i++) {
  const square = document.createElement("input");
  square.classList.add("square");
  square.addEventListener("input", () => {
    solvedMsg.textContent = "";
    square.style.backgroundColor = colours[square.value || 0];
  });
  square.value = initial[i] == 0 ? "" : initial[i];
  square.style.backgroundColor = colours[initial[i]];
  puzzleInput.appendChild(square);
}

// Clear the puzzle input when the 'clear' button is clicked.
document.getElementById("clear").addEventListener("click", () => {
  puzzleInput.childNodes.forEach((node) => {
    node.value = "";
    node.style.backgroundColor = "";
  });
  solvedMsg.textContent = "";
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
  const solved = solve(input, output);

  solvedMsg.textContent = solved ? "Solved puzzle" : "Could not solve";
  solvedMsg.style.color = solved ? "green" : "red";

  // Update inputs.
  puzzleInput.childNodes.forEach((node, i) => {
    node.value = output[i] == 0 ? "" : output[i];
    node.style.backgroundColor = colours[output[i]];
  });
});

document.getElementById("randomSolved").addEventListener("click", () => {
  solvedMsg.textContent = "";
  const output = new Uint8Array(81);
  generate_solved_board(output);
  // Update inputs.
  puzzleInput.childNodes.forEach((node, i) => {
    node.value = output[i] == 0 ? "" : output[i];
    node.style.backgroundColor = colours[output[i]];
  });
});
