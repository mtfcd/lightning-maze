mod utils;
use wasm_bindgen::prelude::*;
use std::collections::HashSet;
use rand::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u32,
    pub y: u32
}

impl Position {
    fn advance(&self, direction: usize, max_width:u32, max_height: u32) -> Option<Self> {
        match direction {
            0 => {
                if self.y == 0 {
                    return None;
                }
                Some(Self{x: self.x, y: self.y - 1})
            },
            1 => {
                if self.y == max_height {
                    return None
                }
                Some(Self{x: self.x, y: self.y + 1})
            }
            2 => {
                if self.x == 0 {
                    return None
                }
                Some(Self{x: self.x - 1, y: self.y})
            },
            3 => {
                if self.x == max_width {
                    return None;
                }
                Some(Self{x: self.x + 1, y: self.y})
            },
            _ => None
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Cell {
    position: Position,
    path: Vec<Position>,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wall {
    Open = 0,
    Block = 1,
}

impl Cell {
    
}

#[wasm_bindgen]
pub struct Maze {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    v_walls: Vec<Wall>,
    h_walls: Vec<Wall>,
    visited: HashSet<Position>,
    winner: Option<Cell>
}

pub type CellWalls = [Wall; 4];

impl Maze {
    fn get_cell_walls(&self, cell: &Cell) -> CellWalls {
        let row = cell.position.y;
        let column = cell.position.x;
        [
            self.h_walls[(row * self.width + column) as usize], // upper
            self.h_walls[((row + 1) * self.width + column) as usize], // down
            self.v_walls[(row * (self.width + 1) + column) as usize], // left
            self.v_walls[(row * (self.width + 1) + column + 1) as usize] // right
        ]
    }

}

fn gen_wall(rng: &mut ThreadRng, p: f32) -> Wall {
    if rng.gen::<f32>() > p {
        Wall::Block
    } else {
        Wall::Open
    }
}

fn gen_all_walls(width: u32, height: u32, px: f32, py: f32) -> (Vec<Wall>, Vec<Wall>) {
    let mut rng = thread_rng();

    let v_walls = (0..(width + 1) * height)
        .map(|i| {
            if i % (width + 1) == 0 || (i + 1) % (width + 1) == 0 {
                return Wall::Block
            }
            gen_wall(&mut rng, px)
        })
        .collect();
    
    let h_walls: Vec<Wall> = (0..(width) * (height + 1))
        .map(|_| {
            gen_wall(&mut rng, py)
        })
        .collect();
    
    (v_walls, h_walls)
}

#[wasm_bindgen]
impl Maze {
    fn split_cell(&mut self, cell: &Cell, positions: &[Position]) -> Vec<Cell> {
        let mut new_cells = Vec::new();
        let new_positions: Vec<Position> = positions.into_iter().filter(|p| !self.visited.contains(p)).map(|p| *p).collect();
        for p in new_positions {
            self.visited.insert(p);
            let mut new_path = cell.path.clone();
            new_path.push(p);
            new_cells.push(Cell {
                position: p,
                path: new_path,
            })
        }

        new_cells
    }
    pub fn tick(&mut self) {
        let mut next = Vec::new();
        let cells = self.cells.clone();
        for cell in cells {
            let walls = self.get_cell_walls(&cell);
            let mut new_positions = Vec::new();
            for (direction, wall) in walls.iter().enumerate() {
                if let Wall::Open = wall {
                    if let Some(p) = cell.position.advance(direction, self.width, self.height) {
                        new_positions.push(p);
                    }
                }
            }

            let mut new_cells = self.split_cell(&cell, &new_positions);
            if let Some(c) = new_cells.iter().find(|c| c.position.y == self.height) {
                self.winner = Some(c.clone());
                self.cells = Vec::new();
                return
            }
            next.append(&mut new_cells);
        }

        self.cells = next;
    }

    pub fn lightup(&self) -> *const u8 {
        if let Some(cell) = &self.winner {
            return cell.path.iter().flat_map(|p| [p.x as u8, p.y as u8]).collect::<Vec<u8>>().as_ptr()
        } else {
            return 0 as *const u8
        }
    }

    pub fn light_path_len(&self) -> u32 {
        self.winner.as_ref().map_or(0, |c| c.path.len() as u32)
    }

    pub fn new(width: u32, height: u32, px: f32, py: f32) -> Maze {
        let (mut v_walls, mut h_walls) = gen_all_walls(width, height, px, py);
        let p = loop {
            if let Some((x, _)) = h_walls.iter().enumerate().skip((width / 2) as usize).find(|(_, w)| **w == Wall::Open) {
                break Position {x: x as u32, y: 0}
            }
            let new_walls = gen_all_walls(width, height, px, py);
            v_walls = new_walls.0;
            h_walls = new_walls.1;
        };
        
        let first_cell = Cell {
            position: p,
            path: vec![p],
        };

        let mut visited = HashSet::new();
        visited.insert(p);
        Maze {
            width,
            height,
            cells: vec![first_cell],
            v_walls,
            h_walls,
            visited,
            winner: None
        }
    }

    pub fn clear_cells(&mut self) {
        let (y, _) = self.h_walls.iter().enumerate().skip((self.width / 2) as usize).find(|(_, w)| **w == Wall::Open).unwrap();
        let p =  Position {x: 0, y: y as u32};
        let first_cell = Cell {
            position: p,
            path: vec![p],
        };

        self.cells = vec![first_cell];
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u8 {
        self.cells.iter().flat_map(|c| [c.position.x as u8, c.position.y as u8]).collect::<Vec<u8>>().as_ptr()
    }

    pub fn cell_count(&self) -> u32 {
        self.cells.len() as u32
    }

    pub fn h_walls(&self) -> *const Wall {
        self.h_walls.as_ptr()
    }
    pub fn v_walls(&self) -> *const Wall {
        self.v_walls.as_ptr()
    }
}