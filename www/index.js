import { Maze, Cell } from "lightning-maze";
import { memory } from "lightning-maze/lightning_maze_bg";

const CELL_SIZE = 4; // px
const BLOCK_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#F0F0F0";
const BACKGROUND_COLOR = "#000000";

let maze = Maze.new(64, 64, 0.4, 0.7);
const width = maze.width();
const height = maze.height();

const sleep = async (mill) => {
  return new Promise((resolve) => setTimeout(resolve, 50));
}

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("lightning-maze-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');
ctx.fillStyle = BACKGROUND_COLOR;
ctx.fillRect(0, 0, canvas.width, canvas.height);

let animationId = null;

const renderLoop = async () => {
  await sleep(10);
  maze.tick();
  const cell_counts = maze.cell_count();
  if (cell_counts == 0) {
    lightup();
    pause();
    return
  }
  drawCells();

  animationId = requestAnimationFrame(renderLoop);
};

const isPaused = () => {
  return animationId === null;
};

const playPauseButton = document.getElementById("play-pause");

const play = () => {
  playPauseButton.textContent = "⏸";
  renderLoop();
};

const pause = () => {
  playPauseButton.textContent = "▶";
  cancelAnimationFrame(animationId);
  animationId = null;
};

playPauseButton.addEventListener("click", event => {
  if (isPaused()) {
    play();
  } else {
    pause();
  }
});

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = BLOCK_COLOR;

  const vWallsPtr = maze.v_walls();
  const vWalls = new Uint8Array(memory.buffer, vWallsPtr, (width + 1) * height);

  for (let i = 0; i <= width + 1; i++) {
    for (let j = 0; j <= height; j++) {
      const cur_x = i * (CELL_SIZE + 1);
      const cur_y = j * (CELL_SIZE + 1);
      if (vWalls[j*(width+1) + i]) {
        ctx.moveTo(cur_x, cur_y);
        ctx.lineTo(cur_x, cur_y + CELL_SIZE + 1);
      }
    }
  }
  
  const hWallsPtr = maze.h_walls();
  const hWalls = new Uint8Array(memory.buffer, hWallsPtr, width * (height + 1));
  for (let i = 0; i <= width; i++) {
    for (let j = 0; j <= height + 1; j++) {
      const cur_x = i * (CELL_SIZE + 1);
      const cur_y = j * (CELL_SIZE + 1);
      if (hWalls[j*width + i]) {
        ctx.moveTo(cur_x,                 cur_y);
        ctx.lineTo(cur_x + CELL_SIZE + 1, cur_y);
      }
    }
  }

  ctx.stroke();
};

const refreshCells = () => {
  ctx.beginPath();

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      ctx.fillStyle = BACKGROUND_COLOR;
      ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }

  ctx.stroke();
};

const getRowRange = (cells) => {
  let max = 0;
  let min = height;

  for (let i = 0; i < cells.length; i++) {
    const row = cells[2 * i + 1];
    if (row < min) {
      min = row;
    }
    if (row > max) {
      max = row;
    }
  }

  return [min, max];
} 

const cell_shade = ["#101", "#323", "#535", "#747", "#959", "#b7b", "#dad", "#FcF"]

const getCellColor = (row, min, max) => {
  const total = cell_shade.length;
  const idx =  Math.ceil((total / (max - min)) * (row - min));
  return cell_shade[idx];
}

const drawCells = () => {
  refreshCells();
  const cellsPtr = maze.cells();
  const cell_counts = maze.cell_count();
  const cells = new Uint8Array(memory.buffer, cellsPtr, cell_counts * 2);
  ctx.beginPath();

  let [min, max] = getRowRange(cells);

  for (let i = 0; i < cell_counts; i++) {
    let col = cells[2 * i];
    let row = cells[2 * i + 1];
    ctx.fillStyle = getCellColor(row, min, max);
    ctx.fillRect(
      col * (CELL_SIZE + 1) + 1,
      row * (CELL_SIZE + 1) + 1,
      CELL_SIZE,
      CELL_SIZE
    );
  }
  
  ctx.stroke();
};

const flash = async (pathWait, cellWait) => {
  refreshCells();
  await sleep(pathWait);
  const cellsPtr = maze.lightup();
  const cell_counts = maze.light_path_len();
  const cells = new Uint8Array(memory.buffer, cellsPtr, cell_counts * 2);
  ctx.beginPath();

  for (let i = 0; i < cell_counts; i++) {
    let col = cells[2 * i];
    let row = cells[2 * i + 1];

    await sleep(cellWait);
    ctx.fillStyle = "#FcF";
    ctx.fillRect(
      col * (CELL_SIZE + 1) + 1,
      row * (CELL_SIZE + 1) + 1,
      CELL_SIZE,
      CELL_SIZE
    );
  }
  
  ctx.stroke();
}

const lightup = async () => {
  flash(50, 1);
  flash(50, 0);
  flash(50, 0);
};


drawGrid();
drawCells();
// requestAnimationFrame(renderLoop);
