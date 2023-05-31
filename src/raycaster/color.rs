use cgmath::Vector2;

#[derive(Clone, Copy)]
pub enum Color {
    Solid([u8; 4]),
    Test,
    Test2,
}

impl Color {
    pub fn sample(&self, pos: Vector2<f64>) -> [u8; 4] {
        match self {
            Self::Solid(color) => *color,
            Self::Test => [(pos.x * 255.0) as u8, (pos.y * 255.0) as u8, 0, 255],
            Self::Test2 => {
                let mut color = [255, 255, 255, 255];
                if pos.x > 0.4 && pos.x < 0.6 && pos.y > 0.4 && pos.y < 0.6 {
                    color[3] = 0;
                }
                color
            }
        }
    }

    pub fn transparent(&self, x: f64) -> bool {
        match self {
            Self::Solid(color) => if color[3] != 255 {true} else {false},
            Self::Test => false,
            Self::Test2 => x > 0.25 && x < 0.75
        }
    }
}
