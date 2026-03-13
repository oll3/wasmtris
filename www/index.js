import { GameContext } from "wasmtris";

const canvas = document.getElementById("wasmtris-canvas");
const _gl = canvas.getContext("webgl", { antialias: false });

const gameContext = GameContext.new("wasmtris-canvas", 16, 30);

function draw() {
	gameContext.draw();
	window.requestAnimationFrame(draw);
}

window.requestAnimationFrame(draw);

// Game mainloop
setInterval(() => {
	gameContext.update(performance.now());
}, 1000.0 / 60.0);
