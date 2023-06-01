use cgmath::Vector2;

#[derive(Clone, Copy)]
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
                if pos.x > 0.4 && pos.x < 0.6 && pos.y > 0.4 && pos.y < 0.6 {
                    color[3] = 0.2;
                }
                color
            }
        }
    }

    pub fn transparent(&self, x: f64) -> bool {
        match self {
            Self::Solid(color) => if color[3] != 1.0 {true} else {false},
            Self::Test => false,
            Self::Test2 => true,
        }
    }
}
