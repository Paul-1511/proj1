use raylib::prelude::{
    Color, Vector2, Vector3, Camera3D, Model, Image, Texture2D, RaylibHandle, RaylibThread, RaylibDrawHandle, KeyboardKey,
    RaylibDraw, RaylibTexture2D, RaylibMode3DExt, RaylibDraw3D
};

mod player;
mod maze;
mod ray;
mod textures;
mod game_state;

use player::{Player, process_events};
use crate::player::check_collision;
use maze::{Maze, CELL_SIZE, Cell, WIDTH, HEIGHT};
use ray::cast_ray;
use textures::TextureManager;
use game_state::{GameState, GameMode};

#[derive(Clone, Copy)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

struct MenuState {
    num_ghosts: usize,
    difficulty: Difficulty,
    selected: usize, // 0 = ghosts, 1 = difficulty, 2 = start
}

const FOV: f32 = std::f32::consts::FRAC_PI_3;
const NUM_RAYS: i32 = 800;

struct Ghost {
    pos: Vector2,
    speed: f32,
    model: Model,
    color: Color,
}

fn spawn_ghosts(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    num_ghosts: usize,
    difficulty: Difficulty,
) -> Vec<Ghost> {
    let mut result = Vec::new();
    let ghost_specs = [
        ("assets/red_ghost.glb", Color::RED, 10.0, 13.0),
        ("assets/yellow_ghost.glb", Color::YELLOW, 10.0, 11.0),
    ];
    let base_speed = match difficulty {
        Difficulty::Easy => 1.0,
        Difficulty::Medium => 2.0,
        Difficulty::Hard => 3.0,
    };
    for i in 0..num_ghosts {
        let (model_path, color, gx, gy) = ghost_specs[i];
        let model = rl.load_model(thread, model_path).expect("Failed to load ghost model");
        result.push(Ghost {
            pos: Vector2::new(
                gx * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0,
                gy * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0,
            ),
            speed: base_speed,
            model,
            color,
        });
    }
    result
}

fn main() {
    if !std::path::Path::new("assets").exists() {
        println!("Error: No se encuentra el directorio 'assets'");
        if let Ok(current_dir) = std::env::current_dir() {
            println!("Directorio actual: {}", current_dir.display());
        }
        return;
    }

    let screen_width = 800;
    let screen_height = 800;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Pac-Man 3D")
        .build();

    rl.set_target_fps(60);

    let mut maze = Maze::new();
    let mut player = Player::new(10, 15);
    let texture_manager = TextureManager::new(&mut rl, &thread);
    let mut game_state = GameState::new(&rl);

    // --- Load animated GIF for moneda.gif as coin frames
    let mut anim_frames = 0;
    let mut coin_textures: Vec<Texture2D> = Vec::new();
    let coin_image = Image::load_image_anim("assets/moneda.gif", &mut anim_frames);
    // SAFE: Only use the first frame (no unsafe pointer manipulation or per-frame copying)
    let tex = rl.load_texture_from_image(&thread, &coin_image).unwrap();
    coin_textures.push(tex);

    // Menu state
    let mut menu_state = MenuState {
        num_ghosts: 2,
        difficulty: Difficulty::Medium,
        selected: 0,
    };
    let mut ghosts: Vec<Ghost> = Vec::new();

    // Start in menu
    game_state.mode = GameMode::Menu;

    while !rl.window_should_close() {
        game_state.update(&rl);
        game_state.handle_input(&rl);

        // Forcibly restart the program after Game Over 5s
        if game_state.is_game_over() {
            if let Some(game_over_time) = game_state.game_over_time {
                if rl.get_time() > game_over_time + 5.0 {
                    // Relaunch the executable and exit this process (hard reboot)
                    std::process::Command::new(std::env::current_exe().unwrap()).spawn().unwrap();
                    std::process::exit(0);
                }
            }
        }

        // MENU
        if game_state.mode == GameMode::Menu {
            if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                menu_state.selected = (menu_state.selected + 1) % 3;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                menu_state.selected = (menu_state.selected + 2) % 3;
            }
            if menu_state.selected == 0 {
                if rl.is_key_pressed(KeyboardKey::KEY_LEFT) || rl.is_key_pressed(KeyboardKey::KEY_ONE) {
                    menu_state.num_ghosts = 1;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) || rl.is_key_pressed(KeyboardKey::KEY_TWO) {
                    menu_state.num_ghosts = 2;
                }
            }
            if menu_state.selected == 1 {
                if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
                    menu_state.difficulty = match menu_state.difficulty {
                        Difficulty::Easy => Difficulty::Hard,
                        Difficulty::Medium => Difficulty::Easy,
                        Difficulty::Hard => Difficulty::Medium,
                    };
                }
                if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
                    menu_state.difficulty = match menu_state.difficulty {
                        Difficulty::Easy => Difficulty::Medium,
                        Difficulty::Medium => Difficulty::Hard,
                        Difficulty::Hard => Difficulty::Easy,
                    };
                }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                ghosts = spawn_ghosts(&mut rl, &thread, menu_state.num_ghosts, menu_state.difficulty);
                maze = Maze::new();
                player = Player::new(10, 15);
                game_state.mode = GameMode::Playing;
            }
        }

        if game_state.is_playing() {
            process_events(&mut player, &maze, &rl, &game_state);

            let grid_x = (player.pos.x / CELL_SIZE as f32) as usize;
            let grid_y = (player.pos.y / CELL_SIZE as f32) as usize;

            if let Some(points) = maze.collect_pellet(grid_x, grid_y) {
                game_state.add_score(points);

                if points == 50 {
                    game_state.activate_power_mode();
                }
            }

            if maze.is_level_complete() {
                game_state.complete_level(&rl);
            }

            // Move ghosts toward player
            for ghost in ghosts.iter_mut() {
                ghost.speed = player.speed * 0.75;
                let dx = player.pos.x - ghost.pos.x;
                let dy = player.pos.y - ghost.pos.y;
                let mut moved = false;
                if dx.abs() > dy.abs() {
                    let step_x = ghost.speed * dx.signum();
                    let next_x = ghost.pos.x + step_x;
                    if !check_collision(next_x, ghost.pos.y, &maze) {
                        ghost.pos.x = next_x;
                        moved = true;
                    }
                    if !moved {
                        let step_y = ghost.speed * dy.signum();
                        let next_y = ghost.pos.y + step_y;
                        if !check_collision(ghost.pos.x, next_y, &maze) {
                            ghost.pos.y = next_y;
                        }
                    }
                } else {
                    let step_y = ghost.speed * dy.signum();
                    let next_y = ghost.pos.y + step_y;
                    if !check_collision(ghost.pos.x, next_y, &maze) {
                        ghost.pos.y = next_y;
                        moved = true;
                    }
                    if !moved {
                        let step_x = ghost.speed * dx.signum();
                        let next_x = ghost.pos.x + step_x;
                        if !check_collision(next_x, ghost.pos.y, &maze) {
                            ghost.pos.x = next_x;
                        }
                    }
                }
                if (ghost.pos.x - player.pos.x).abs() < CELL_SIZE as f32 / 2.0 && (ghost.pos.y - player.pos.y).abs() < CELL_SIZE as f32 / 2.0 {
                    game_state.mode = game_state::GameMode::GameOver;
                }
            }
        } else if game_state.is_level_complete() {
            if game_state.is_playing() {
                maze = Maze::new();
                player = Player::new(10, 15);
                ghosts = spawn_ghosts(&mut rl, &thread, menu_state.num_ghosts, menu_state.difficulty);
            }
        }

        let anim_speed = 10.0;
        let frame_idx = if !coin_textures.is_empty() {
            ((rl.get_time() * anim_speed) as usize) % coin_textures.len()
        } else { 0 };

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        if game_state.mode == GameMode::Menu {
            d.draw_text("PAC-MAN 3D", 260, 100, 40, Color::YELLOW);
            let ghost_str = format!("Number of Ghosts: {}", menu_state.num_ghosts);
            let diff_str = format!("Difficulty: {}", match menu_state.difficulty {
                Difficulty::Easy => "Easy",
                Difficulty::Medium => "Medium",
                Difficulty::Hard => "Hard",
            });
            let start_str = "Start Game";
            let menu_items = [ghost_str.as_str(), diff_str.as_str(), start_str];
            for (i, item) in menu_items.iter().enumerate() {
                let color = if i == menu_state.selected { Color::GREEN } else { Color::WHITE };
                d.draw_text(item, 300, 200 + i as i32 * 40, 30, color);
            }
            d.draw_text("Use UP/DOWN to select, LEFT/RIGHT or 1/2 to change, ENTER to start", 100, 400, 18, Color::LIGHTGRAY);
        }

        if game_state.is_playing() || game_state.is_paused() {
            for i in 0..NUM_RAYS {
                let ray_angle = player.angle - FOV / 2.0 + FOV * (i as f32) / (NUM_RAYS as f32);
                let intersect = cast_ray(player.pos, ray_angle, &maze);
                let corrected_distance = intersect.distance * (player.angle - ray_angle).cos();
                let wall_height = (CELL_SIZE as f32 * screen_height as f32) / corrected_distance;
                let texture_id = texture_manager.get_wall_texture_id(intersect.wall_x, intersect.wall_y);

                if let Some(texture) = texture_manager.textures.get(&texture_id) {
                    let tex_width = texture.width() as f32;
                    let tex_height = texture.height() as f32;
                    let tex_x = (intersect.texture_coord * tex_width).clamp(0.0, tex_width - 1.0);
                    let source_rect = raylib::core::math::Rectangle {
                        x: tex_x,
                        y: 0.0,
                        width: 1.0,
                        height: tex_height,
                    };
                    let dest_rect = raylib::core::math::Rectangle {
                        x: i as f32,
                        y: (screen_height as f32 / 2.0) - wall_height / 2.0,
                        width: 1.0,
                        height: wall_height,
                    };
                    d.draw_texture_pro(
                        texture,
                        source_rect,
                        dest_rect,
                        raylib::core::math::Vector2 { x: 0.0, y: 0.0 },
                        0.0,
                        Color::WHITE,
                    );
                } else {
                    let y_start = (screen_height / 2 - (wall_height / 2.0) as i32).max(0);
                    let y_end = (screen_height / 2 + (wall_height / 2.0) as i32).min(screen_height);
                    for y in y_start..y_end {
                        d.draw_pixel(i, y, Color::RED);
                    }
                }
            }

            // Animated coin rendering
            if !coin_textures.is_empty() {
                let coin_draw_size = (coin_textures[0].width().max(coin_textures[0].height()) as f32) * 0.5;
                let moneda_tex = &coin_textures[frame_idx];
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        match maze.get_cell(x, y) {
                            Cell::Pellet | Cell::PowerPellet => {
                                let fx = x as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0;
                                let fy = y as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0;
                                let rel_x = fx - player.pos.x;
                                let rel_y = fy - player.pos.y;
                                let angle_to_point = rel_y.atan2(rel_x) - player.angle;
                                let dist = (rel_x*rel_x + rel_y*rel_y).sqrt();
                                if angle_to_point.abs() < FOV / 2.0 && dist > (CELL_SIZE as f32)/2.0 && dist < 500.0 {
                                    let mut blocked = false;
                                    let steps = dist.max(1.0) as i32 / 8;
                                    for step in 1..steps {
                                        let t = step as f32 / steps as f32;
                                        let cx = player.pos.x + (fx - player.pos.x) * t;
                                        let cy = player.pos.y + (fy - player.pos.y) * t;
                                        let ix = (cx / CELL_SIZE as f32).floor() as usize;
                                        let iy = (cy / CELL_SIZE as f32).floor() as usize;
                                        if ix < WIDTH && iy < HEIGHT {
                                            if matches!(maze.get_cell(ix, iy), Cell::Wall) {
                                                blocked = true;
                                                break;
                                            }
                                        }
                                    }
                                    if !blocked {
                                        let proj = screen_width as f32 / 2.0 + angle_to_point / (FOV/2.0) * (screen_width as f32 / 2.0) * 0.7;
                                        let vpos = (screen_height as f32 / 2.0) + (CELL_SIZE as f32*70.0/dist).clamp(-220.0,220.0);
                                        let tex_rec = raylib::core::math::Rectangle {
                                            x: 0.0,
                                            y: 0.0,
                                            width: moneda_tex.width() as f32,
                                            height: moneda_tex.height() as f32,
                                        };
                                        let out_rec = raylib::core::math::Rectangle {
                                            x: proj - coin_draw_size/2.0,
                                            y: vpos - coin_draw_size/2.0,
                                            width: coin_draw_size,
                                            height: coin_draw_size,
                                        };
                                        d.draw_texture_pro(moneda_tex, tex_rec, out_rec, raylib::core::math::Vector2{x:0.0,y:0.0}, 0.0, Color::WHITE);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            let camera = Camera3D::perspective(
                Vector3::new(player.pos.x, 10.0, player.pos.y),
                Vector3::new(
                    player.pos.x + player.angle.cos() * 10.0,
                    0.0,
                    player.pos.y + player.angle.sin() * 10.0,
                ),
                Vector3::new(0.0, 1.0, 0.0),
                60.0,
            );
            {
                let mut mode3d = d.begin_mode3D(&camera);
                for ghost in ghosts.iter() {
                    let ghost_pos_3d = Vector3::new(ghost.pos.x, 2.0, ghost.pos.y - 2.0);
                    mode3d.draw_model(&ghost.model, ghost_pos_3d, 1.0, ghost.color);
                }
            }
            render_minimap(&mut d, &maze, &player, &ghosts, &texture_manager, screen_width);
        }

        game_state.draw_ui(&mut d, screen_width, screen_height);
    }

    if game_state.is_game_over() {
        let allow_buttons = if let Some(game_over_time) = game_state.game_over_time {
            rl.get_time() < game_over_time + 5.0
        } else { false };
        if allow_buttons {
            if rl.is_key_pressed(KeyboardKey::KEY_R) {
                ghosts = spawn_ghosts(&mut rl, &thread, menu_state.num_ghosts, menu_state.difficulty);
                game_state.reset_game(&rl);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_M) {
                game_state.mode = GameMode::Menu;
            }
        }
    }
}

fn render_minimap(
    d: &mut RaylibDrawHandle,
    maze: &Maze,
    player: &Player,
    ghosts: &Vec<Ghost>,
    _texture_manager: &TextureManager,
    screen_width: i32,
) {
    let ghost_colors = [Color::RED, Color::YELLOW];
    let minimap_size = 200;
    let cell_size = minimap_size / WIDTH.max(HEIGHT) as i32;
    let offset_x = screen_width - minimap_size - 10;
    let offset_y = 10;
    d.draw_rectangle(
        offset_x - 5,
        offset_y - 5,
        minimap_size + 10,
        minimap_size + 10,
        Color::new(0, 0, 0, 180),
    );
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let draw_x = offset_x + x as i32 * cell_size;
            let draw_y = offset_y + y as i32 * cell_size;
            match maze.get_cell(x, y) {
                Cell::Wall => {
                    d.draw_rectangle(draw_x, draw_y, cell_size, cell_size, Color::BLUE);
                }
                Cell::Pellet => {
                    d.draw_rectangle(draw_x, draw_y, cell_size, cell_size, Color::BLACK);
                    d.draw_circle(draw_x + cell_size / 2, draw_y + cell_size / 2, 2.0, Color::YELLOW);
                }
                Cell::PowerPellet => {
                    d.draw_rectangle(draw_x, draw_y, cell_size, cell_size, Color::BLACK);
                    d.draw_circle(draw_x + cell_size / 2, draw_y + cell_size / 2, 4.0, Color::YELLOW);
                }
                Cell::Path => {
                    d.draw_rectangle(draw_x, draw_y, cell_size, cell_size, Color::BLACK);
                }
            }
        }
    }
    let grid_x = (player.pos.x / CELL_SIZE as f32).round() as i32;
    let grid_y = (player.pos.y / CELL_SIZE as f32).round() as i32;
    let player_x = offset_x + grid_x * cell_size;
    let player_y = offset_y + grid_y * cell_size;
    d.draw_circle(player_x + cell_size / 2, player_y + cell_size / 2, 6.0, Color::RED);
    for (i, ghost) in ghosts.iter().enumerate() {
        let ghost_grid_x = (ghost.pos.x / CELL_SIZE as f32).round() as i32;
        let ghost_grid_y = (ghost.pos.y / CELL_SIZE as f32).round() as i32;
        let ghost_x = offset_x + ghost_grid_x * cell_size;
        let ghost_y = offset_y + ghost_grid_y * cell_size;
        let color = if i < ghost_colors.len() { ghost_colors[i] } else { Color::PINK };
        d.draw_circle(ghost_x + cell_size / 2, ghost_y + cell_size / 2, 6.0, color);
    }
    let dir_x = player_x + cell_size / 2 + (player.angle.cos() * 8.0) as i32;
    let dir_y = player_y + cell_size / 2 + (player.angle.sin() * 8.0) as i32;
    d.draw_line(
        player_x + cell_size / 2,
        player_y + cell_size / 2,
        dir_x,
        dir_y,
        Color::RED,
    );
}
