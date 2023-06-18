use cgmath::Vector2;

pub mod camera;
pub use camera::*;

pub mod map;
pub use map::*;

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
                    left -= self.render_wall(x, &wall_hit, camera, map.wall_height);
                    left == 0
                }
                Hit::FloorHit(floor_hit) => {
                    left -= self.render_floor(x, &floor_hit, camera);
                    left -= self.render_ceiling(x, &floor_hit, camera);

                    left == 0
                }
            });

            x += 1;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let index1 = x * self.height + y;
                let index2 = (y * self.width + x) * 4;
                for i in 0..3 {
                    screen[index2 + i] = (self.temp_screen[index1][i] * 255.0) as u8;
                }
                screen[index2 + 3] = 255;
            }
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: [f64; 4]) -> bool {
        let index = x * self.height + y;
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
        let index = x * self.height + y;
        if self.temp_screen[index][3] == 0.0 {
            true
        } else {
            false
        }
    }

    fn render_wall(
        &mut self,
        x: usize,
        wall_hit: &WallHit,
        camera: &Camera,
        wall_height: f64,
    ) -> usize {
        let line_height = (self.height as f64 / wall_hit.length * wall_height) as i32;
        let mid_point = (self.height as i32) / 2
            + ((camera.z() * 2.0 - wall_height) * self.height as f64 / (2.0 * wall_hit.length))
                as i32;
        //println!("{} {} {}", camera.z(), line_height, mid_point);
        let start = -line_height / 2 + mid_point;
        let end = line_height / 2 + mid_point;

        let draw_start = std::cmp::min(std::cmp::max(start, 0), self.height as i32) as usize;
        let draw_end = std::cmp::min(std::cmp::max(end, 0), self.height as i32) as usize;

        let mut drawn = 0;
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

    fn render_floor(&mut self, x: usize, floor_hit: &FloorHit, camera: &Camera) -> usize {
        let z = -camera.z() * 2.0 + 1.0 + floor_hit.floor_height * 2.0;
        let start = self.y_from_floor_dist(floor_hit.dist2, z);
        let end = self.y_from_floor_dist(floor_hit.dist1, z);
        let h = self.height as f64;

        let mut drawn: usize = 0;
        for y in start..end {
            let current_dist = h * (1.0 - z) / (2.0 * (y as f64) - h);
            if !self.pixel_finished(x, y) {
                let weight = (current_dist - floor_hit.dist1) / (floor_hit.dist2 - floor_hit.dist1);
                let floor_pos = weight * floor_hit.pos2 + (1.0 - weight) * floor_hit.pos1;
                let color = floor_hit.floor_color.sample(floor_pos);
                if self.set_pixel(x, y, color) {
                    drawn += 1;
                }
            }
        }
        drawn
    }

    fn render_ceiling(&mut self, x: usize, floor_hit: &FloorHit, camera: &Camera) -> usize {
        let z = -camera.z() * 2.0 - 1.0 + floor_hit.ceiling_height * 2.0;
        let start = self.y_from_ceiling_dist(floor_hit.dist1, z);
        let end = self.y_from_ceiling_dist(floor_hit.dist2, z);
        let h = self.height as f64;

        let mut drawn = 0;
        for y in start..end {
            let current_dist = h * (z + 1.0) / (h - 2.0 * (y as f64));
            if !self.pixel_finished(x, y) {
                let weight = (current_dist - floor_hit.dist1) / (floor_hit.dist2 - floor_hit.dist1);
                let floor_pos = weight * floor_hit.pos2 + (1.0 - weight) * floor_hit.pos1;
                let color = floor_hit.ceiling_color.sample(floor_pos);
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
            std::cmp::min(
                (self.height as f64 * (dist - z - 1.0) / (2.0 * dist)) as usize,
                self.height / 2,
            )
        }
    }
}
