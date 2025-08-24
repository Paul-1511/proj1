use raylib::prelude::*;
use std::collections::HashMap;

pub struct TextureManager {
    pub textures: HashMap<u8, Texture2D>,
}

impl TextureManager {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut textures = HashMap::new();

        // Lista de texturas a cargar
        let texture_files = vec![
            (1, "assets/pacmanmap.png"),
            (2, "assets/moneda.gif"), // Points asset
        ];

        for (id, path) in texture_files {
            if !std::path::Path::new(path).exists() {
                println!("Error: No se encuentra el archivo {}", path);
                continue;
            }

            match Image::load_image(path) {
                Ok(image) => {
                    println!(
                        "Textura {} cargada: {}x{} píxeles",
                        id, image.width, image.height
                    );

                    match rl.load_texture_from_image(&thread, &image) {
                        Ok(texture) => {
                            textures.insert(id, texture);
                            println!("Textura {} convertida a Texture2D correctamente", id);
                        }
                        Err(e) => {
                            println!("Error al crear textura desde imagen {}: {:?}", path, e);
                        }
                    }
                }
                Err(e) => {
                    println!("Error al cargar imagen {}: {:?}", path, e);
                }
            }
        }

        if textures.is_empty() {
            println!("Advertencia: No se cargaron texturas, usando colores de fallback");
        }

        TextureManager { textures }
    }

    pub fn get_pixel_color(&self, texture_id: u8, _tx: u32, _ty: u32) -> Color {
        // Fallback color only, as we cannot safely access image data
        match texture_id {
            1 => Color::RED,
            2 => Color::BLUE,
            3 => Color::GREEN,
            _ => Color::WHITE,
        }
    }

    pub fn get_wall_texture_id(&self, _wall_x: usize, _wall_y: usize) -> u8 {
        // Por ahora, siempre devuelve el ID 1
        1
    }
}
