use std::error::Error;

use cgmath::{Matrix2, Vector2};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer, Rgba};

pub mod shape;
pub use shape::*;

pub mod color;
pub use color::*;

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

const MAX_HITS: usize = 8; 
pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = Vec::new();
        tiles.resize(
            width * height,
            Tile::new(Shape::Void, vec![])
        );
        Self {
            width,
            height,
            tiles,
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x >= self.width {
            panic!("x: {} is outside the range [0, {})", x, self.width);
        }
        if y >= self.height {
            panic!("y: {} is outside the range [0, {})", y, self.height);
        }

        self.tiles[y * self.width + x] = tile;
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Tile {
        if x >= self.width {
            panic!("x: {} is outside the range [0, {})", x, self.width);
        }
        if y >= self.height {
            panic!("y: {} is outside the range [0, {})", y, self.height);
        }

        self.tiles[y * self.width + x]
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>, hit_callback: &mut dyn FnMut(Hit) -> bool) {
        let mut map_pos: Vector2<i32> = pos.cast().unwrap();
        let delta_dist = dir.map(|a| 1.0 / a.abs());

        let mut side_dist = Vector2::new(
            if dir.x < 0.0 {
                (pos.x - map_pos.x as f64) * delta_dist.x
            } else {
                (map_pos.x as f64 + 1.0 - pos.x) * delta_dist.x
            },
            if dir.y < 0.0 {
                (pos.y - map_pos.y as f64) * delta_dist.y
            } else {
                (map_pos.y as f64 + 1.0 - pos.y) * delta_dist.y
            },
        );

        let step = dir.map(|a| if a < 0.0 { -1 } else { 1 });

        let hit_tile = self.get_tile(map_pos.x as usize, map_pos.y as usize);
        let tile_pos = pos - map_pos.cast().unwrap();
        if let Some(shape_info) = hit_tile.shape.ray_cast(tile_pos, dir) {
            let hit_info = Hit::WallHit(WallHit { 
                length: shape_info.length,
                x: shape_info.x,
                color: hit_tile.colors[shape_info.side as usize],
            });
            if hit_callback(hit_info) {return};
        }

        let hit = false;
        let mut side;

        while !hit {
            let mut tile_pos = pos;
            if side_dist.x < side_dist.y {
                tile_pos += dir * side_dist.x;
                side_dist.x += delta_dist.x;
                map_pos.x += step.x;
                side = 0;
            } else {
                tile_pos += dir * side_dist.y;
                side_dist.y += delta_dist.y;
                map_pos.y += step.y;
                side = 1;
            }
            tile_pos -= map_pos.cast().unwrap();
            //println!("{:?}", map_pos);
            let hit_tile = self.get_tile(map_pos.x as usize, map_pos.y as usize);
            if let Some(shape_info) = hit_tile.shape.ray_cast(tile_pos, dir) {
                let perp_wall_dist = if side == 0 {
                    side_dist.x - delta_dist.x
                } else {
                    side_dist.y - delta_dist.y
                };
                let hit_info = Hit::WallHit(WallHit {
                    length: shape_info.length + perp_wall_dist,
                    x: shape_info.x,
                    color: hit_tile.colors[shape_info.side as usize],
                });
                if hit_callback(hit_info) {return;};
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Hit {
    WallHit(WallHit)
}

#[derive(Clone, Copy)]
struct WallHit {
    pub length: f64,
    pub x: f64,
    pub color: Color,
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub shape: Shape,
    pub colors: [Color; 4],
}

impl Tile {
    pub fn new(shape: Shape, colors: Vec<Color>) -> Self {
        if colors.len() as u32 != shape.sides() {
            panic!("Wrong number of colors");
        }
        let mut tile = Self {
            shape, colors: [Color::Test; 4]
        };
        for i in 0..colors.len() {
            tile.colors[i] = colors[i];
        }

        tile
    }
}

pub struct Renderer {
    width: usize,
    height: usize,
    temp_screen: Vec<[f64; 4]>,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut temp_screen = Vec::new();
        temp_screen.resize(width*height, [0.0; 4]);
        Self { width, height, temp_screen }
    }

    pub fn render(&mut self, screen: &mut [u8], camera: &Camera, map: &Map) {
        for i in 0..self.width*self.height {
            self.temp_screen[i] = [0.0, 0.0, 0.0, 1.0];
        }

        let mut x = 0;
        let pos = camera.pos();
        for ray_dir in camera.rays(self.width as u32) {
            let mut left = self.height;
            map.ray_cast(pos, ray_dir, &mut |hit| {
                match hit {
                    Hit::WallHit(wall_hit) => {
                        let line_height = (self.width as f64 / wall_hit.length) as i32;
                        let start = -line_height / 2 + (self.height as i32) / 2;
                        let end = line_height / 2 + (self.height as i32) / 2;

                        let draw_start = std::cmp::max(start, 0) as usize;
                        let draw_end = std::cmp::min(end, self.height as i32) as usize;

                        for y in draw_start..draw_end {
                            let index = y * self.width + x;
                            if self.temp_screen[index][3] > 0.0 {
                                let color = wall_hit.color.sample(Vector2 {
                                    x: wall_hit.x,
                                    y: ((y as i32 - start) as f64) / ((end - start) as f64),
                                });
                                for i in 0..3 {
                                    self.temp_screen[index][i] += self.temp_screen[index][3]*color[3]*color[i];
                                }
                                self.temp_screen[index][3] *= 1.0-color[3];
                                if self.temp_screen[index][3] == 0.0 {
                                    if left != 0 { left -= 1; };
                                }
                            }
                        }
                        true
                    }
                }
            });

            x += 1;
        }

        for i in 0..self.width*self.height {
            for j in 0..3 {
                screen[i*4+j] = (self.temp_screen[i][j]*255.0) as u8;
            }
            screen[i*4+3] = 255;
        }
    }
}