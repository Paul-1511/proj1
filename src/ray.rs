use raylib::math::Vector2;
use crate::maze::{Maze, Cell, CELL_SIZE};

pub struct Intersect {
    pub distance: f32,
    pub wall_x: usize,
    pub wall_y: usize,
    pub texture_coord: f32,
}

pub fn cast_ray(origin: Vector2, angle: f32, maze: &Maze) -> Intersect {
    let mut distance = 0.0;
    let step = 0.5;
    let mut wall_x = 0;
    let mut wall_y = 0;
    let mut target_x = 0.0;
    let mut target_y = 0.0;

    loop {
        distance += step;
        target_x = origin.x + angle.cos() * distance;
        target_y = origin.y + angle.sin() * distance;

        let grid_x = (target_x / CELL_SIZE as f32) as usize;
        let grid_y = (target_y / CELL_SIZE as f32) as usize;

        if grid_x >= maze.grid[0].len() || grid_y >= maze.grid.len() {
            wall_x = grid_x;
            wall_y = grid_y;
            break;
        }

        if maze.grid[grid_y][grid_x] == Cell::Wall {
            wall_x = grid_x;
            wall_y = grid_y;
            break;
        }

        if distance > 1000.0 {
            break;
        }
    }

    // Cálculo mejorado de coordenadas de textura
    let wall_cell_x = wall_x as f32 * CELL_SIZE as f32;
    let wall_cell_y = wall_y as f32 * CELL_SIZE as f32;

    let dx = target_x - wall_cell_x;
    let dy = target_y - wall_cell_y;

    let texture_coord = if dx.abs() > dy.abs() {
        // Pared vertical
        (target_y - wall_cell_y) / CELL_SIZE as f32
    } else {
        // Pared horizontal
        (target_x - wall_cell_x) / CELL_SIZE as f32
    }.fract().abs(); // Aseguramos valor entre 0 y 1

    Intersect {
        distance,
        wall_x,
        wall_y,
        texture_coord,
    }
}