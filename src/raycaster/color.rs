use cgmath::Vector2;

#[derive(Clone, Copy, Debug)]
pub enum Color {
    Solid([f64; 4]),
    Test,
    Test2,
}

impl Color {
    pub fn sample(&self, pos: Vector2<f64>) -> [f64; 4] {
        match self {
            Self::Solid(color) => *color,
            Self::Test => [pos.x, pos.y, 0.0, 1.0],
            Self::Test2 => {
                let mut color = [1.0; 4];
                if pos.x > 0.2 && pos.x < 0.8 && pos.y > 0.2 && pos.y < 0.8 {
                    color = [0.5, 0.5, 1.0, 0.3];
                }
                color
            }
        }
    }

    pub fn transparent(&self, x: f64) -> bool {
        match self {
            Self::Solid(color) => {
                if color[3] != 1.0 {
                    true
                } else {
                    false
                }
            }
            Self::Test => false,
            Self::Test2 => true,
        }
    }
}
