import * as wasm from "wasmtris";

const canvas = document.getElementById("wasmtris-canvas");

let playfield = createPlayfield(10, 20);
let blockSize = 64;

// resize the canvas to fill browser window dynamically
function resizeCanvas() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;

  const maxBlockWidth = window.innerWidth / playfield[0].length;
  const maxBlockHeight = canvas.height / playfield.length;
  blockSize = Math.min(maxBlockWidth, maxBlockHeight);
  draw();
}
window.addEventListener("resize", resizeCanvas, false);
resizeCanvas();

function createPlayfield(width, height) {
  let playfield = [];
  for (let y = 0; y < height; y++) {
    let row = [];
    for (let x = 0; x < width; x++) {
      row.push(true);
    }
    playfield.push(row);
  }
  return playfield;
}

function drawBlock(ctx, x, y, size) {
  const x1 = size * x;
  const x2 = x1 + size;
  const y1 = size * y;
  const y2 = y1 + size;
  ctx.fillRect(x1, y1, size, size);
  ctx.strokeRect(x1, y1, size, size);
}

function drawPlayfield(pf) {
  if (canvas.getContext) {
    const ctx = canvas.getContext("2d");
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.strokeStyle = "rgba(170, 190, 180, 1)";
    ctx.fillStyle = "rgba(20, 20, 30, 1)";
    ctx.lineWidth = 2;
    for (let y = 0; y < pf.length; y++) {
      for (let x = 0; x < pf[y].length; x++) {
        if (pf[y][x]) {
          drawBlock(ctx, x, y, blockSize);
        }
      }
    }
  }
}

function draw() {
  if (playfield) {
    drawPlayfield(playfield);
  }
}
