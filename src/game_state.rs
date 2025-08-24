use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    Playing,
    LevelComplete,
    GameOver,
    Paused,
    Menu,
}

pub struct GameState {
    pub score: u32,
    pub level: u32,
    pub lives: u32,
    pub mode: GameMode,
    pub high_score: u32,
    pub level_start_time: f64,
    pub game_time: f64,
    pub level_complete_time: f64,
    pub power_mode_active: bool,
    pub power_mode_timer: f64,
    pub power_mode_duration: f64,
    pub bonus_multiplier: u32,
    pub game_over_time: Option<f64>, // Added for GameOver timer
}

impl GameState {
    pub fn new(rl: &RaylibHandle) -> Self {
        GameState {
            score: 0,
            level: 1,
            lives: 3,
            mode: GameMode::Playing,
            high_score: 0,
            level_start_time: rl.get_time(),
            game_time: 0.0,
            level_complete_time: 0.0,
            power_mode_active: false,
            power_mode_timer: 0.0,
            power_mode_duration: 10.0,
            bonus_multiplier: 1,
            game_over_time: None,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        let current_time = rl.get_time();
        self.game_time = current_time - self.level_start_time;

        if self.power_mode_active {
            self.power_mode_timer -= rl.get_frame_time() as f64;
            if self.power_mode_timer <= 0.0 {
                self.deactivate_power_mode();
            }
        }

        self.update_bonus_multiplier();
    }

    pub fn add_score(&mut self, points: u32) {
        let bonus_points = points * self.bonus_multiplier;
        self.score += bonus_points;
        
        if self.score > self.high_score {
            self.high_score = self.score;
        }

        if self.score > 0 && self.score % 10000 == 0 {
            self.lives += 1;
            println!("¡Vida extra! Total: {}", self.lives);
        }
    }

    pub fn activate_power_mode(&mut self) {
        self.power_mode_active = true;
        self.power_mode_timer = self.power_mode_duration;
        self.bonus_multiplier = 2;
        println!("¡Modo de poder activado!");
    }

    pub fn deactivate_power_mode(&mut self) {
        self.power_mode_active = false;
        self.power_mode_timer = 0.0;
        self.bonus_multiplier = 1;
    }

    pub fn lose_life(&mut self, rl: &RaylibHandle) {
        if self.lives > 0 {
            self.lives -= 1;
        }
        
        if self.lives == 0 {
            self.mode = GameMode::GameOver;
            self.game_over_time = Some(rl.get_time());
            println!("Game Over! Puntuación final: {}", self.score);
        }
    }

    pub fn complete_level(&mut self, rl: &RaylibHandle) {
        self.mode = GameMode::LevelComplete;
        self.level_complete_time = rl.get_time();
        
        let time_bonus = ((120.0 - self.game_time).max(0.0) * 10.0) as u32;
        let level_bonus = self.level * 1000;
        
        self.add_score(level_bonus + time_bonus);
        
        println!("¡Nivel {} completado!", self.level);
        println!("Bonificación de tiempo: {}", time_bonus);
        println!("Bonificación de nivel: {}", level_bonus);
    }

    pub fn next_level(&mut self, rl: &RaylibHandle) {
        self.level += 1;
        self.mode = GameMode::Playing;
        self.level_start_time = rl.get_time();
        self.deactivate_power_mode();
        
        println!("Comenzando nivel {}", self.level);
    }

    pub fn reset_game(&mut self, rl: &RaylibHandle) {
        self.score = 0;
        self.level = 1;
        self.lives = 3;
        self.mode = GameMode::Playing;
        self.level_start_time = rl.get_time();
        self.game_time = 0.0;
        self.deactivate_power_mode();
        self.bonus_multiplier = 1;
        self.game_over_time = None;
    }

    pub fn pause_game(&mut self) {
        if self.mode == GameMode::Playing {
            self.mode = GameMode::Paused;
        } else if self.mode == GameMode::Paused {
            self.mode = GameMode::Playing;
        }
    }

    pub fn is_playing(&self) -> bool {
        self.mode == GameMode::Playing
    }

    pub fn is_paused(&self) -> bool {
        self.mode == GameMode::Paused
    }

    pub fn is_game_over(&self) -> bool {
        self.mode == GameMode::GameOver
    }

    pub fn is_level_complete(&self) -> bool {
        self.mode == GameMode::LevelComplete
    }

    pub fn get_power_mode_remaining(&self) -> f64 {
        if self.power_mode_active {
            self.power_mode_timer
        } else {
            0.0
        }
    }

    pub fn get_level_time(&self) -> f64 {
        self.game_time
    }

    fn update_bonus_multiplier(&mut self) {
        if !self.power_mode_active {
            if self.game_time < 30.0 {
                self.bonus_multiplier = 3;
            } else if self.game_time < 60.0 {
                self.bonus_multiplier = 2;
            } else {
                self.bonus_multiplier = 1;
            }
        }
    }

    pub fn draw_ui(&self, d: &mut RaylibDrawHandle, screen_width: i32, screen_height: i32) {
        d.draw_text(&format!("SCORE: {:06}", self.score), 10, 10, 24, Color::YELLOW);
        d.draw_text(&format!("HIGH: {:06}", self.high_score), 10, 40, 16, Color::ORANGE);

        if self.bonus_multiplier > 1 {
            d.draw_text(&format!("BONUS x{}", self.bonus_multiplier), 
                       screen_width - 120, 10, 20, Color::GREEN);
        }

        if self.power_mode_active {
            let remaining = self.get_power_mode_remaining() as i32;
            d.draw_text(&format!("POWER: {}s", remaining), 
                       screen_width - 120, 40, 18, Color::YELLOW);

            let bar_width = 100;
            let bar_height = 8;
            let progress = (self.power_mode_timer / self.power_mode_duration) as f32;
            
            d.draw_rectangle(screen_width - 120, 65, bar_width, bar_height, Color::DARKGRAY);
            d.draw_rectangle(screen_width - 120, 65, 
                           (bar_width as f32 * progress) as i32, bar_height, Color::YELLOW);
        }

        let minutes = (self.game_time / 60.0) as i32;
        let seconds = (self.game_time % 60.0) as i32;
        d.draw_text(&format!("TIME: {:02}:{:02}", minutes, seconds), 
                   screen_width - 120, 90, 16, Color::LIGHTGRAY);

        match self.mode {
            GameMode::LevelComplete => {
                d.draw_rectangle(0, 0, screen_width, screen_height, Color::new(0, 0, 0, 128));
                d.draw_text("YOU WIN", 
                           screen_width / 2 - 80, screen_height / 2 - 20, 40, Color::GREEN);
                d.draw_text("Press M for Menu", 
                           screen_width / 2 - 90, screen_height / 2 + 30, 24, Color::WHITE);
            }
            GameMode::GameOver => {
                d.draw_rectangle(0, 0, screen_width, screen_height, Color::new(0, 0, 0, 128));
                d.draw_text("GAME OVER", 
                           screen_width / 2 - 80, screen_height / 2 - 40, 32, Color::RED);
                d.draw_text(&format!("Final Score: {}", self.score), 
                           screen_width / 2 - 70, screen_height / 2, 20, Color::WHITE);
                // Only show R/M instructions if <5s since game over
                if let Some(game_over_time) = self.game_over_time {
                    // WARNING: d.get_shader_time() was invalid, substitute a constant because drawing logic uses UI timer, not absolute clock.
                    // In practice, this timer logic is used in the main game loop.
                    // Always draw the text when in the first 5 seconds after game over (handled at input/menu logic)
                    // So just always show it when GameOver and game_over_time is set.
                    d.draw_text("Press R to Retry or M for Menu", 
                               screen_width / 2 - 150, screen_height / 2 + 30, 20, Color::LIGHTGRAY);
                }
            }
            GameMode::Paused => {
                d.draw_rectangle(0, 0, screen_width, screen_height, Color::new(0, 0, 0, 128));
                d.draw_text("PAUSED", 
                           screen_width / 2 - 60, screen_height / 2, 32, Color::WHITE);
                d.draw_text("Press P to resume", 
                           screen_width / 2 - 70, screen_height / 2 + 40, 18, Color::LIGHTGRAY);
            }
            _ => {}
        }
    }

    pub fn handle_input(&mut self, rl: &RaylibHandle) {
        match self.mode {
            GameMode::Playing => {
                if rl.is_key_pressed(KeyboardKey::KEY_P) {
                    self.pause_game();
                }
            }
            GameMode::GameOver => {
                // Ignore inputs here; handled in main.rs
            }
            GameMode::Paused => {
                if rl.is_key_pressed(KeyboardKey::KEY_P) {
                    self.pause_game();
                }
            }
            GameMode::LevelComplete => {
                if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    self.next_level(rl);
                }
            }
            _ => {}
        }
    }
}