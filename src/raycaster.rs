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
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = Vec::<Tile>::new();
        tiles.resize(width * height, Tile::VOID);
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

    pub fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>) -> f64 {
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

        let mut hit = false;
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
            let hit_tile = self.get_tile(map_pos.x as usize, map_pos.y as usize);
            if let Tile::WALL(wall) = hit_tile {
                hit = true;
            }
        }

        let perp_wall_dist = if side == 0 {
            side_dist.x - delta_dist.x
        } else {
            side_dist.y - delta_dist.y
        };
        perp_wall_dist
    }
}

#[derive(Clone, Copy)]
pub enum Tile {
    VOID,
    WALL([u8; 3]),
}

pub fn render(screen: &mut [u8], width: usize, height: usize, camera: &Camera, map: &Map) {
    let mut x = 0;
    let pos = camera.pos();
    for ray_dir in camera.rays(width as u32) {
        let dist = map.ray_cast(pos, ray_dir);

        let line_height = (width as f64 / dist) as i32;
        let draw_start = std::cmp::max(0, -line_height / 2 + (height as i32) / 2);
        let draw_end = std::cmp::min(height as i32, line_height / 2 + (height as i32) / 2);

        for y in 0..height {
            let index = (y * width + x) * 4;
            if draw_start <= y as i32 && y as i32 <= draw_end {
                for i in 0..3 {
                    screen[index + i] = 0xff;
                }
            } else {
                for i in 0..3 {
                    screen[index + i] = 0x00;
                }
                screen[index + 3] = 0xff;
            }
            screen[index + 3] = 0xff;
        }

        x += 1;
    }
}
