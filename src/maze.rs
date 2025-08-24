use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Wall,
    Path,
    Pellet,
    PowerPellet,
}

pub const CELL_SIZE: i32 = 32;
pub const WIDTH: usize = 21;
pub const HEIGHT: usize = 25;

pub struct Maze {
    pub grid: Vec<Vec<Cell>>,
    pellets_collected: u32,
    total_pellets: u32,
}

impl Maze {
    pub fn new() -> Self {
        let mut grid = vec![vec![Cell::Wall; WIDTH]; HEIGHT];
        let mut total_pellets = 0;
        
        // Laberinto básico estilo Pac-Man
        let layout = [
            "#####################",
            "#.........#.........#",
            "#o##.###.#.#.###.##o#",
            "#...................#",
            "#.##.#.#####.#.##.#.#",
            "#....#...#...#....#.#",
            "####.###.#.###.######",
            "   #.#.......#.#     ",
            "####.#.##-##.#.######",
            "#......#   #......  #",
            "####.#.#####.#.######",
            "   #.#.......#.#     ",
            "####.#.#####.#.######",
            "#.........#.........#",
            "#.##.###.#.#.###.##.#",
            "#o.#.....P.....#.o.#",
            "##.#.#.#####.#.#.##.#",
            "#....#...#...#....#.#",
            "#.######.#.######.#.#",
            "#...................#",
            "#####################",
        ];

        for (y, row) in layout.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if y < HEIGHT && x < WIDTH {
                    grid[y][x] = match ch {
                        '#' => Cell::Wall,
                        '.' => {
                            total_pellets += 1;
                            Cell::Pellet
                        },
                        'o' => {
                            total_pellets += 1;
                            Cell::PowerPellet
                        },
                        'P' => Cell::Path, // Posición inicial del jugador
                        _ => Cell::Path,
                    };
                }
            }
        }

        Maze {
            grid,
            pellets_collected: 0,
            total_pellets,
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Cell {
        if y < self.grid.len() && x < self.grid[0].len() {
            self.grid[y][x]
        } else {
            Cell::Wall
        }
    }

    pub fn collect_pellet(&mut self, x: usize, y: usize) -> Option<u32> {
        if y < self.grid.len() && x < self.grid[0].len() {
            match self.grid[y][x] {
                Cell::Pellet => {
                    self.grid[y][x] = Cell::Path;
                    self.pellets_collected += 1;
                    Some(10)
                },
                Cell::PowerPellet => {
                    self.grid[y][x] = Cell::Path;
                    self.pellets_collected += 1;
                    Some(50)
                },
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn is_level_complete(&self) -> bool {
        self.pellets_collected >= self.total_pellets
    }
}

pub fn get_pixel_color(image: &Image, x: i32, y: i32) -> Color {
    let width = image.width as usize;
    let height = image.height as usize;

    if x < 0 || y < 0 || x as usize >= width || y as usize >= height {
        return Color::new(0, 0, 255, 255);
    }

    let x = x as usize;
    let y = y as usize;
    let data_len = width * height * 4;

    unsafe {
        let data = std::slice::from_raw_parts(image.data as *const u8, data_len);
        let idx = (y * width + x) * 4;

        if idx + 3 >= data_len {
            return Color::new(0, 0, 255, 255);
        }

        Color::new(data[idx], data[idx + 1], data[idx + 2], data[idx + 3])
    }
}
