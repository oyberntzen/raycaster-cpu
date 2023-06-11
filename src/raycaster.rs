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
    z: f64,
}

impl Camera {
    pub fn new(pos: Vector2<f64>, rot: f64, fov: f64) -> Self {
        Self {
            pos,
            dir_front: Vector2::new(rot.cos(), rot.sin()),
            dir_right: Vector2::new(rot.sin(), rot.cos()),
            plane: Vector2::new(rot.sin(), rot.cos()) * (fov / 2.0).tan(),
            z: 0.0
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

    pub fn translate_z(&mut self, delta: f64) {
        self.z += delta;
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

    pub fn z(&self) -> f64 {
        self.z
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
    wall_height: f64
}

impl Map {
    pub fn new(width: usize, height: usize, wall_height: f64) -> Self {
        let mut tiles = Vec::new();
        tiles.resize(
            width * height,
            Tile::new(Shape::Void, vec![], Color::Test, 0.0, Color::Test, wall_height),
        );
        Self {
            width,
            height,
            tiles,
            wall_height
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

    pub fn get_tile(&self, x: i32, y: i32) -> Option<Tile> {
        if x < 0 || x >= self.width as i32 {
            return None
        }
        if y < 0 || y >= self.height as i32 {
            return None
        }

        Some(self.tiles[y as usize * self.width + x as usize])
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

        let hit_tile = self.get_tile(map_pos.x, map_pos.y);
        if let Some(tile) = hit_tile {
            let tile_pos = pos - map_pos.cast().unwrap();
            if let Some(shape_info) = tile.shape.ray_cast(tile_pos, dir) {
                let hit_info = Hit::WallHit(WallHit {
                    length: shape_info.length,
                    x: shape_info.x,
                    color: &tile.colors[shape_info.side as usize],
                });
                if hit_callback(hit_info) {
                    return;
                };
            }
        } else {
            return
        }

        let hit = false;
        let mut side;
        let mut last_pos = pos;
        let mut last_map_pos;
        let mut dist;
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

            let hit_tile = self.get_tile(last_map_pos.x, last_map_pos.y);
            if let Some(tile) = hit_tile {
                let floor_hit = FloorHit {
                    pos1: last_pos - last_map_pos.cast().unwrap(),
                    pos2: tile_pos - last_map_pos.cast().unwrap(),
                    dist1: last_dist,
                    dist2: dist,
                    floor_color: &tile.floor_color,
                    floor_height: tile.floor_height,
                    ceiling_color: &tile.ceiling_color,
                    ceiling_height: tile.ceiling_height
                };
                if hit_callback(Hit::FloorHit(floor_hit)) {
                    return;
                }
            } else {
                return;
            }
            last_pos = tile_pos;
            last_dist = dist;

            tile_pos -= map_pos.cast().unwrap();
            //println!("{:?}", map_pos);
            let hit_tile = self.get_tile(map_pos.x, map_pos.y);
            if let Some(tile) = hit_tile {
                if let Some(shape_info) = tile.shape.ray_cast(tile_pos, dir) {
                    let perp_wall_dist = if side == 0 {
                        side_dist.x - delta_dist.x
                    } else {
                        side_dist.y - delta_dist.y
                    };
                    let hit_info = Hit::WallHit(WallHit {
                        length: shape_info.length + perp_wall_dist,
                        x: shape_info.x,
                        color: &tile.colors[shape_info.side as usize],
                    });
                    if hit_callback(hit_info) {
                        return;
                    };
                }
            }
        }
    }
}

enum Hit<'a> {
    WallHit(WallHit<'a>),
    FloorHit(FloorHit<'a>),
}

struct WallHit<'a> {
    length: f64,
    x: f64,
    color: &'a Color,
}

struct FloorHit<'a> {
    pos1: Vector2<f64>,
    pos2: Vector2<f64>,
    dist1: f64,
    dist2: f64,
    floor_color: &'a Color,
    floor_height: f64,
    ceiling_color: &'a Color,
    ceiling_height: f64
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub shape: Shape,
    pub colors: [Color; 4],
    pub floor_color: Color,
    pub floor_height: f64,
    pub ceiling_color: Color,
    pub ceiling_height: f64,
}

impl Tile {
    pub fn new(shape: Shape, colors: Vec<Color>, floor_color: Color, floor_height: f64, ceiling_color: Color, ceiling_height: f64) -> Self {
        if colors.len() as u32 != shape.sides() {
            panic!("Wrong number of colors");
        }
        let mut tile = Self {
            shape,
            colors: [Color::Test; 4],
            floor_color,
            floor_height,
            ceiling_color,
            ceiling_height,
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
                    left -= self.render_wall(x, wall_hit, camera, map.wall_height);
                    left == 0
                }
                Hit::FloorHit(floor_hit) => {
                    let z = floor_hit.floor_height * 2.0 - camera.z();
                    let start = self.y_from_floor_dist(floor_hit.dist2, z);
                    let end = self.y_from_floor_dist(floor_hit.dist1, z);
                    let h = self.height as f64;

                    for y in start..end {
                        let current_dist = h * (1.0 - z) / (2.0 * (y as f64) - h);
                        if !self.pixel_finished(x, y) {
                            let weight =
                                (current_dist - floor_hit.dist1) / (floor_hit.dist2 - floor_hit.dist1);
                            let floor_pos = weight * floor_hit.pos2 + (1.0 - weight) * floor_hit.pos1;
                            let color = floor_hit.floor_color.sample(floor_pos);
                            if self.set_pixel(x, y, color) && left != 0 {
                                left -= 1;
                            }
                        }
                    }

                    let z = floor_hit.ceiling_height * 2.0 - camera.z() - 2.0;
                    let start = self.y_from_ceiling_dist(floor_hit.dist1, z);
                    let end = self.y_from_ceiling_dist(floor_hit.dist2, z);
                    let h = self.height as f64;

                    for y in start..end {
                        let current_dist = h * (z + 1.0) / (h - 2.0 * (y as f64));
                        if !self.pixel_finished(x, y) {
                            let weight =
                                (current_dist - floor_hit.dist1) / (floor_hit.dist2 - floor_hit.dist1);
                            let floor_pos = weight * floor_hit.pos2 + (1.0 - weight) * floor_hit.pos1;
                            let color = floor_hit.ceiling_color.sample(floor_pos);
                            if self.set_pixel(x, y, color) && left != 0 {
                                left -= 1;
                            }
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
        for i in 0..3 {
            self.temp_screen[index][i] += self.temp_screen[index][3] * color[3] * color[i];
        }
        self.temp_screen[index][3] *= 1.0 - color[3];
        if self.temp_screen[index][3] == 0.0 {
            true
        } else {
            false
        }
    }

    fn pixel_finished(&self, x: usize, y: usize) -> bool {
        let index = y * self.width + x;
        if self.temp_screen[index][3] == 0.0 {
            true
        } else {
            false
        }
    }

    fn render_wall(&mut self, x: usize, wall_hit: WallHit, camera: &Camera, wall_height: f64) -> usize {
        let line_height = (self.height as f64 / wall_hit.length * wall_height) as i32;
        let mid_point = (self.height as i32) / 2 + (((camera.z() - wall_height/2.0) * self.height as f64 / 2.0) / wall_hit.length) as i32;
        let start = -line_height / 2 + mid_point;
        let end = line_height / 2 + mid_point;

        let draw_start = std::cmp::max(start, 0) as usize;
        let draw_end = std::cmp::min(end, self.height as i32) as usize;

        let mut drawn: usize = 0;
        for y in draw_start..draw_end {
            if !self.pixel_finished(x, y) {
                let color = wall_hit.color.sample(Vector2 {
                    x: wall_hit.x,
                    y: ((y as i32 - start) as f64) / ((end - start) as f64),
                });
                if self.set_pixel(x, y, color) {
                    drawn += 1;
                }
            }
        }
        drawn
    }

    fn y_from_floor_dist(&self, dist: f64, z: f64) -> usize {
        if dist == 0.0 {
            self.height
        } else {
            std::cmp::min(
                ((self.height as f64 * (dist - z + 1.0)) / (2.0 * dist)) as usize,
                self.height,
            )
        }
    }

    fn y_from_ceiling_dist(&self, dist: f64, z: f64) -> usize {
        if dist == 0.0 {
            0
        } else {
            std::cmp::min((self.height as f64 * (dist - z - 1.0) / (2.0 * dist)) as usize, self.height/2)
        }
    }
}
