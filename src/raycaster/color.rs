use cgmath::Vector2;

#[derive(Clone, Copy)]
pub enum Color {
    Solid([u8; 3]),
    Test,
}

impl Color {
    pub fn sample(&self, pos: Vector2<f64>) -> [u8; 3] {
        match self {
            Color::Solid(color) => *color,
            Color::Test => [(pos.x * 255.0) as u8, (pos.y * 255.0) as u8, 0],
        }
    }
}
