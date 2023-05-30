use cgmath::{Vector2};

#[derive(Clone, Copy)]
pub enum Shape {
    VOID,
    BOX,
    AXIS_ALIGNED_BOX(AxisAlignedBox),
}

impl Shape {
    pub fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>) -> Option<f64> {
        match self {
            Self::VOID => None,
            Self::BOX => Some(0.0),
            Self::AXIS_ALIGNED_BOX(shape) => shape.ray_cast(pos, dir),
        }
    }
}

#[derive(Clone, Copy)]
pub struct AxisAlignedBox {
    pub min: Vector2<f64>,
    pub max: Vector2<f64>
}

impl AxisAlignedBox {
    fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>) -> Option<f64> {
        let x = if dir.x > 0.0 { self.min.x } else { self.max.x };
        let a = (x - pos.x) / dir.x;
        let y = a * dir.y + pos.y;
        if y >= self.min.y && y <= self.max.y && a >= 0.0 { return Some(a) };

        let y = if dir.y > 0.0 { self.min.y } else { self.max.y };
        let a = (y - pos.y) / dir.y;
        let x = a * dir.x + pos.x;
        if x >= self.min.x && x <= self.max.x && a >= 0.0 { return Some(a) };

        None
    }
}