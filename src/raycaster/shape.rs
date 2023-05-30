use cgmath::{Vector2};

#[derive(Clone, Copy)]
pub enum Shape {
    Void,
    Box,
    AxisAlignedBox(AxisAlignedBox),
    Circle(Circle),
}

impl Shape {
    pub fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>) -> Option<f64> {
        match self {
            Self::Void => None,
            Self::Box => Some(0.0),
            Self::AxisAlignedBox(shape) => shape.ray_cast(pos, dir),
            Self::Circle(shape) => shape.ray_cast(pos, dir),
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

#[derive(Clone, Copy)]
pub struct Circle {
    pub pos: Vector2<f64>,
    pub radius: f64,
}

impl Circle {
    fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>) -> Option<f64> {
        let a = dir.x*dir.x + dir.y*dir.y;
        let b = 2.0 * (dir.x*(pos.x-self.pos.x) + dir.y*(pos.y-self.pos.y));
        let k = pos - self.pos;
        let c = k.x*k.x + k.y*k.y - self.radius*self.radius;

        let in_sqrt = b*b - 4.0*a*c;
        if in_sqrt > 0.0 {
            let sqrt = in_sqrt.sqrt();
            let l1 = (-b - sqrt) / (2.0 * a);
            let l2 = (-b + sqrt) / (2.0 * a);
            if l1 >= 0.0 {
                Some(l1)
            } else if l2 >= 0.0 {
                Some(l2)
            } else {
                None
            }
        } else {
            None
        }
    }
}