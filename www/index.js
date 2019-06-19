import { GameContext } from "wasmtris";

const canvas = document.getElementById("wasmtris-canvas");


const gameContext = GameContext.new("wasmtris-canvas", 16, 30);

// resize the canvas to fill browser window dynamically
function resizeCanvas() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;

  const maxBlockWidth = window.innerWidth / 16;
  const maxBlockHeight = canvas.height / 30;
  //blockSize = Math.min(maxBlockWidth, maxBlockHeight);
  gameContext.draw();
}
window.addEventListener("resize", resizeCanvas, false);
resizeCanvas();

// Game mainloop
setInterval(() => {
  // Update game state // performance.now()
  gameContext.update(performance.now());

  gameContext.draw();

}, 1000.0 / 60.0)