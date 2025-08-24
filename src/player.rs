use raylib::prelude::*;
use crate::maze::{Maze, Cell, CELL_SIZE};
use crate::game_state::GameState;

pub struct Player {
    pub pos: Vector2,
    pub angle: f32,
    pub speed: f32,
    pub rotation_speed: f32,
}

impl Player {
    pub fn new(grid_x: usize, grid_y: usize) -> Self {
        let x = (grid_x as i32 * CELL_SIZE + CELL_SIZE / 2) as f32;
        let y = (grid_y as i32 * CELL_SIZE + CELL_SIZE / 2) as f32;
        Self {
            pos: Vector2::new(x, y),
            angle: 0.0,
            speed: 2.5,
            rotation_speed: 0.05,
        }
    }
}

pub fn process_events(player: &mut Player, maze: &Maze, rl: &RaylibHandle, game_state: &GameState) {
    // Set speed based on power mode
    if game_state.power_mode_active {
        player.speed = 2.5 * 1.5;
    } else {
        player.speed = 2.5;
    }
    let mut dir = Vector2::new(0.0, 0.0);

    if rl.is_key_down(KeyboardKey::KEY_D) {
        player.angle += player.rotation_speed;
    }
    if rl.is_key_down(KeyboardKey::KEY_A) {
        player.angle -= player.rotation_speed;
    }

    if rl.is_key_down(KeyboardKey::KEY_W) {
        dir.x += player.angle.cos();
        dir.y += player.angle.sin();
    }
    if rl.is_key_down(KeyboardKey::KEY_S) {
        dir.x -= player.angle.cos();
        dir.y -= player.angle.sin();
    }

    if dir.length() > 0.0 {
        dir = dir.normalized(); 
        let next_x = player.pos.x + dir.x * player.speed;
        let next_y = player.pos.y + dir.y * player.speed;

        if !check_collision(next_x, player.pos.y, maze) {
            player.pos.x = next_x;
        }
        if !check_collision(player.pos.x, next_y, maze) {
            player.pos.y = next_y;
        }
    }
}

pub fn check_collision(x: f32, y: f32, maze: &Maze) -> bool {
    let grid_x = (x / CELL_SIZE as f32) as usize;
    let grid_y = (y / CELL_SIZE as f32) as usize;

    if grid_y >= maze.grid.len() || grid_x >= maze.grid[0].len() {
        return true;
    }

    maze.grid[grid_y][grid_x] == Cell::Wall
}
