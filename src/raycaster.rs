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
        tiles.resize(width * height, Tile::new(Shape::Void, vec![], Color::Test, Color::Test));
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

    fn ray_cast(
        &self,
        pos: Vector2<f64>,
        dir: Vector2<f64>,
        hit_callback: &mut dyn FnMut(Hit) -> bool,
    ) {
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
            if hit_callback(hit_info) {
                return;
            };
        }

        let hit = false;
        let mut side;
        let mut last_pos = pos;
        let mut last_map_pos = map_pos;
        let mut dist = 0.0;
        let mut last_dist = 0.0;

        while !hit {
            let mut tile_pos = pos;
            last_map_pos = map_pos;
            if side_dist.x < side_dist.y {
                dist = side_dist.x;
                tile_pos += dir * side_dist.x;
                side_dist.x += delta_dist.x;
                map_pos.x += step.x;
                side = 0;
            } else {
                dist = side_dist.y;
                tile_pos += dir * side_dist.y;
                side_dist.y += delta_dist.y;
                map_pos.y += step.y;
                side = 1;
            }

            let tile = self.get_tile(last_map_pos.x as usize, last_map_pos.y as usize);
            let floor_hit = FloorHit {
                pos1: last_pos-last_map_pos.cast().unwrap(),
                pos2: tile_pos-last_map_pos.cast().unwrap(),
                dist1: last_dist,
                dist2: dist,
                floor_color: tile.floor_color,
                ceiling_color: tile.ceiling_color,
            };
            hit_callback(Hit::FloorHit(floor_hit));
            last_pos = tile_pos;
            last_dist = dist;

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
                if hit_callback(hit_info) {
                    return;
                };
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Hit {
    WallHit(WallHit),
    FloorHit(FloorHit)
}

#[derive(Clone, Copy, Debug)]
struct WallHit {
    length: f64,
    x: f64,
    color: Color,
}

#[derive(Clone, Copy, Debug)]
struct FloorHit {
    pos1: Vector2<f64>,
    pos2: Vector2<f64>,
    dist1: f64,
    dist2: f64,
    floor_color: Color,
    ceiling_color: Color,
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub shape: Shape,
    pub colors: [Color; 4],
    pub floor_color: Color,
    pub ceiling_color: Color,
}

impl Tile {
    pub fn new(shape: Shape, colors: Vec<Color>, floor_color: Color, ceiling_color: Color) -> Self {
        if colors.len() as u32 != shape.sides() {
            panic!("Wrong number of colors");
        }
        let mut tile = Self {
            shape,
            colors: [Color::Test; 4],
            floor_color,
            ceiling_color
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
        temp_screen.resize(width * height, [0.0; 4]);
        Self {
            width,
            height,
            temp_screen,
        }
    }

    pub fn render(&mut self, screen: &mut [u8], camera: &Camera, map: &Map) {
        for i in 0..self.width * self.height {
            self.temp_screen[i] = [0.0, 0.0, 0.0, 1.0];
        }

        let mut x = 0;
        let pos = camera.pos();
        for ray_dir in camera.rays(self.width as u32) {
            let mut left = self.height;
            map.ray_cast(pos, ray_dir, &mut |hit| match hit {
                Hit::WallHit(wall_hit) => {
                    let line_height = (self.height as f64 / wall_hit.length) as i32;
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
                            if self.set_pixel(x, y, color) && left != 0 {
                                left -= 1;
                            }
                        }
                    }
                    left == 0
                }
                Hit::FloorHit(floor_hit) => {
                    let h = self.height as f64;
                    let start = if floor_hit.dist2 == 0.0  { self.height } else {std::cmp::min((h*(1.0+floor_hit.dist2)/(2.0*floor_hit.dist2)) as usize, self.height)};
                    let end = if floor_hit.dist1 == 0.0  { self.height } else {std::cmp::min((h*(1.0+floor_hit.dist1)/(2.0*floor_hit.dist1)) as usize, self.height)};

                    //println!("{} {}", start, end);
                    for y in start..end {
                        let current_dist = h / (2.0 * (y as f64) - h);
                        let weight = (current_dist - floor_hit.dist1) / (floor_hit.dist2 - floor_hit.dist1);
                        let floor_pos = weight * floor_hit.pos2 + (1.0 - weight) * floor_hit.pos1;
                        let color1 = floor_hit.floor_color.sample(floor_pos);
                        let color2 = floor_hit.ceiling_color.sample(floor_pos);
                        if self.set_pixel(x, y, color1) && left != 0 {
                            left -= 1;
                        }
                        if self.set_pixel(x, self.height-1-y, color2) && left != 0 {
                            left -= 1;
                        }
                    }
                    left == 0
                }
            });

            x += 1;
        }

        for i in 0..self.width * self.height {
            for j in 0..3 {
                screen[i * 4 + j] = (self.temp_screen[i][j] * 255.0) as u8;
            }
            screen[i * 4 + 3] = 255;
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: [f64; 4]) -> bool {
        let index = y * self.width + x;
        if self.temp_screen[index][3] == 0.0 {
            return false
        }
        for i in 0..3 {
            self.temp_screen[index][i] +=
            self.temp_screen[index][3] * color[3] * color[i];
        }
        self.temp_screen[index][3] *= 1.0 - color[3];
        if self.temp_screen[index][3] == 0.0 {
            true
        } else {
            false
        }
    }
}
