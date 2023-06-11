use cgmath::Vector2;

pub mod shape;
pub use shape::*;

pub mod color;
pub use color::*;

pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    pub wall_height: f64
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

    pub fn ray_cast(
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

pub enum Hit<'a> {
    WallHit(WallHit<'a>),
    FloorHit(FloorHit<'a>),
}

pub struct WallHit<'a> {
    pub length: f64,
    pub x: f64,
    pub color: &'a Color,
}

pub struct FloorHit<'a> {
    pub pos1: Vector2<f64>,
    pub pos2: Vector2<f64>,
    pub dist1: f64,
    pub dist2: f64,
    pub floor_color: &'a Color,
    pub floor_height: f64,
    pub ceiling_color: &'a Color,
    pub ceiling_height: f64
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