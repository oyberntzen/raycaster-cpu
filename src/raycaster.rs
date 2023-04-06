use std::error::Error;

use cgmath::{Matrix2, Vector2};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer, Rgba};

pub struct Camera {
    pos: Vector2<f64>,
    dir_front: Vector2<f64>,
    dir_right: Vector2<f64>,
    plane: Vector2<f64>,
}

impl Camera {
    pub fn new(pos: Vector2<f64>, rot: f64, fov: f64) -> Self {
        println!("{}", (fov / 2.0).tan());
        Self {
            pos,
            dir_front: Vector2::new(rot.cos(), rot.sin()),
            dir_right: Vector2::new(rot.sin(), rot.cos()),
            plane: Vector2::new(rot.sin(), rot.cos()) * (fov / 2.0).tan(),
        }
    }

    pub fn rotate(&mut self, radians: f64) {
        let rot_cos = radians.cos();
        let rot_sin = radians.sin();
        let rot_mat = Matrix2::new(rot_cos, rot_sin, -rot_sin, rot_cos);
        self.dir_front = rot_mat * self.dir_front;
        self.dir_right = rot_mat * self.dir_right;
        self.plane = rot_mat * self.plane;
    }

    pub fn translate(&mut self, delta: Vector2<f64>) {
        self.pos += delta.x * self.dir_right + delta.y * self.dir_front;
    }

    pub fn rays(&self, width: u32) -> RayIterator {
        RayIterator {
            current_x: 0,
            width,
            camera: &self,
        }
    }

    pub fn pos(&self) -> Vector2<f64> {
        self.pos
    }
}

pub struct RayIterator<'a> {
    current_x: u32,
    width: u32,
    camera: &'a Camera,
}

impl Iterator for RayIterator<'_> {
    type Item = Vector2<f64>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_x < self.width {
            let camera_x = 2.0 * (self.current_x as f64) / (self.width as f64) - 1.0;
            let ray_dir = self.camera.dir_front + self.camera.plane * camera_x;
            self.current_x += 1;
            Some(ray_dir)
        } else {
            None
        }
    }
}

pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    textures: Vec<Texture>,
    map: Vec<usize>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut map = Vec::<usize>::new();
        map.resize(width * height, 0usize);
        Self {
            width,
            height,
            tiles: Vec::new(),
            textures: Vec::new(),
            map,
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: usize) {
        if x >= self.width {
            panic!("x: {} is outside the range [0, {})", x, self.width);
        }
        if y >= self.height {
            panic!("y: {} is outside the range [0, {})", y, self.height);
        }

        self.map[y * self.width + x] = tile;
    }

    pub fn get_tile(&self, x: usize, y: usize) -> usize {
        if x >= self.width {
            panic!("x: {} is outside the range [0, {})", x, self.width);
        }
        if y >= self.height {
            panic!("y: {} is outside the range [0, {})", y, self.height);
        }

        self.map[y * self.width + x]
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn new_texture(&mut self, path: &str) -> usize {
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let width = img.width();
        let height = img.height();
        let data = if let DynamicImage::ImageRgba8(rgba8) = img {
            rgba8.into_raw()
        } else {
            let rgba8 = img.as_rgba8().unwrap();
            let mut pixels = Vec::<u8>::new();
            for pixel in rgba8.pixels() {
                for i in 0..4 {
                    pixels.push(pixel[i]);
                }
            }
            pixels
        };
        let texture = Texture {
            width: width as usize,
            height: height as usize,
            data,
        };
        self.textures.push(texture);
        self.textures.len() - 1
    }

    pub fn new_tile(&mut self, tile: Tile) -> usize {
        self.tiles.push(tile);
        self.tiles.len()
    }
}

#[derive(Clone, Debug)]
struct Texture {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

pub struct Tile {
    pub color: WallColor,
}

#[derive(Clone, Copy, Debug)]
pub enum WallColor {
    SOLID([u8; 3]),
    TEXTURE(usize),
}

impl WallColor {
    pub fn width(&self, map: &Map) -> usize {
        match self {
            WallColor::SOLID(_) => 1,
            WallColor::TEXTURE(index) => map.textures[*index].width,
        }
    }

    pub fn height(&self, map: &Map) -> usize {
        match self {
            WallColor::SOLID(_) => 1,
            WallColor::TEXTURE(index) => map.textures[*index].height,
        }
    }

    pub fn get_color(&self, map: &Map, x: usize, y: usize) -> [u8; 3] {
        match self {
            WallColor::SOLID(color) => *color,
            WallColor::TEXTURE(index) => {
                let width = self.width(map);
                let pixel_index = (y*width+x)*4;
                let mut color = [0u8; 3];
                for i in 0..3 {
                    color[i] = map.textures[*index].data[pixel_index + i];
                }
                color
            }
        }
    }
}
/*
pub trait Sampler {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn color(&self, x: u32, y: u32) -> [u8; 3];
}

pub struct SolidSampler {
    solid_color: [u8; 3],
}

impl SolidSampler {
    pub fn new(solid_color: [u8; 3]) -> Self {
        Self { solid_color }
    }
}

impl Sampler for SolidSampler {
    fn width(&self) -> u32 {
        1
    }

    fn height(&self) -> u32 {
        1
    }

    fn color(&self, _: u32, _: u32) -> [u8; 3] {
        self.solid_color
    }
}

pub struct TextureSampler<'a> {
    width: u32,
    height: u32,
    data: &'a Vec<u8>,
}

impl<'a> TextureSampler<'a> {
    pub fn from_data(width: u32, height: u32, data: &'a Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }
    pub fn from_map_texture(map: &'a Map, texture_index: u32) -> Self {
        if texture_index >= map.textures.len() as u32 {
            panic!(
                "texture: {} is outside the range [0, {})",
                texture_index,
                map.textures.len()
            );
        }
        let texture = &map.textures[texture_index as usize];
        Self {
            width: texture.width,
            height: texture.height,
            data: &texture.data,
        }
    }
}

impl Sampler for TextureSampler<'_> {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn color(&self, x: u32, y: u32) -> [u8; 3] {
        let mut color = [0u8; 3];
        for i in 0..3 {
            color[i] = self.data[(x * self.width + y) as usize + i];
        }
        color
    }
}
*/
pub fn render(screen: &mut [u8], width: usize, height: usize, camera: &Camera, map: &Map) {
    let mut x = 0;
    let pos = camera.pos();
    for ray_dir in camera.rays(width as u32) {
        let mut map_pos: Vector2<i32> = pos.cast().unwrap();
        let delta_dist = ray_dir.map(|a| 1.0 / a.abs());

        let mut side_dist = Vector2::new(
            if ray_dir.x < 0.0 {
                (pos.x - map_pos.x as f64) * delta_dist.x
            } else {
                (map_pos.x as f64 + 1.0 - pos.x) * delta_dist.x
            },
            if ray_dir.y < 0.0 {
                (pos.y - map_pos.y as f64) * delta_dist.y
            } else {
                (map_pos.y as f64 + 1.0 - pos.y) * delta_dist.y
            },
        );

        let step = ray_dir.map(|a| if a < 0.0 { -1 } else { 1 });

        let mut hit = false;
        let mut hit_wall = 0usize;
        let mut side = 0;

        while !hit {
            if side_dist.x < side_dist.y {
                side_dist.x += delta_dist.x;
                map_pos.x += step.x;
                side = 0;
            } else {
                side_dist.y += delta_dist.y;
                map_pos.y += step.y;
                side = 1;
            }
            let hit_tile = map.get_tile(map_pos.x as usize, map_pos.y as usize);
            if hit_tile != 0 {
                hit = true;
                hit_wall = hit_tile;
            }
        }

        let perp_wall_dist = if side == 0 {
            side_dist.x - delta_dist.x
        } else {
            side_dist.y - delta_dist.y
        };

        let line_height = (width as f64 / perp_wall_dist) as i32;
        let draw_start = std::cmp::max(0, -line_height / 2 + (height as i32) / 2);
        let draw_end = std::cmp::min(height as i32, line_height / 2 + (height as i32) / 2);

        let mut wall_x = if side == 0 {
            pos.y + perp_wall_dist * ray_dir.y
        } else {
            pos.x + perp_wall_dist * ray_dir.x
        };
        wall_x -= wall_x.floor();

        let wall_color = map.tiles[hit_wall-1].color;
        let texture_width = wall_color.width(map);
        let texture_height = wall_color.height(map);
        let mut texture_x = (wall_x * texture_width as f64) as usize;
        if (side == 0 && ray_dir.x > 0.0) || (side == 1 && ray_dir.y > 0.0) {
            texture_x = texture_width - texture_x - 1;
        }
        let texture_step = texture_height as f64 / line_height as f64;
        let mut texture_pos =
            (draw_start as f64 - height as f64 / 2.0 + line_height as f64 / 2.0) * texture_step;

        for y in 0..height {
            let index = (y * width + x) * 4;
            if draw_start <= y as i32 && y as i32 <= draw_end {
                let texture_y = std::cmp::min(texture_pos as usize, texture_height - 1);
                texture_pos += texture_step;
                let color = wall_color.get_color(&map, texture_x, texture_y);
                for i in 0..3 {
                    screen[index + i] = color[i];
                }
                screen[index + 3] = 0xff;
            } else {
                for i in 0..3 {
                    screen[index + i] = 0x00;
                }
                screen[index + 3] = 0xff;
            }
        }

        x += 1;
    }
}
