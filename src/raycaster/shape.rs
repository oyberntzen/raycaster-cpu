use std::f64::consts::PI;

use cgmath::{InnerSpace, Vector2};

#[derive(Clone, Copy)]
pub enum Shape {
    Void,
    Box,
    AxisAlignedBox(AxisAlignedBox),
    Circle(Circle),
    Line(Line),
}

impl Shape {
    pub fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>) -> Option<ShapeHitInfo> {
        match self {
            Self::Void => None,
            Self::Box => {
                const b: AxisAlignedBox = AxisAlignedBox {
                    min: Vector2 { x: 0.0, y: 0.0 },
                    max: Vector2 { x: 1.0, y: 1.0 },
                };
                b.ray_cast(pos, dir)
            }
            Self::AxisAlignedBox(shape) => shape.ray_cast(pos, dir),
            Self::Circle(shape) => shape.ray_cast(pos, dir),
            Self::Line(shape) => shape.ray_cast(pos, dir),
        }
    }
}

pub struct ShapeHitInfo {
    pub length: f64,
    pub x: f64,
    pub side: u32,
}

#[derive(Clone, Copy)]
pub struct AxisAlignedBox {
    pub min: Vector2<f64>,
    pub max: Vector2<f64>,
}

impl AxisAlignedBox {
    fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>) -> Option<ShapeHitInfo> {
        let (x, side) = if dir.x > 0.0 {
            (self.min.x, 0)
        } else {
            (self.max.x, 1)
        };
        let a = (x - pos.x) / dir.x;
        let y = a * dir.y + pos.y;
        if y >= self.min.y && y <= self.max.y && a + 0.001 >= 0.0 {
            return Some(ShapeHitInfo {
                length: a,
                x: (y - self.min.y) / (self.max.y - self.min.y),
                side,
            });
        };

        let (y, side) = if dir.y > 0.0 {
            (self.min.y, 2)
        } else {
            (self.max.y, 3)
        };
        let a = (y - pos.y) / dir.y;
        let x = a * dir.x + pos.x;
        if x >= self.min.x && x <= self.max.x && a + 0.001 >= 0.0 {
            return Some(ShapeHitInfo {
                length: a,
                x: (x - self.min.x) / (self.max.x - self.min.x),
                side,
            });
        };

        None
    }
}

#[derive(Clone, Copy)]
pub struct Circle {
    pub pos: Vector2<f64>,
    pub radius: f64,
}

impl Circle {
    fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>) -> Option<ShapeHitInfo> {
        let a = dir.x * dir.x + dir.y * dir.y;
        let b = 2.0 * (dir.x * (pos.x - self.pos.x) + dir.y * (pos.y - self.pos.y));
        let k = pos - self.pos;
        let c = k.x * k.x + k.y * k.y - self.radius * self.radius;

        let in_sqrt = b * b - 4.0 * a * c;
        if in_sqrt > 0.0 {
            let sqrt = in_sqrt.sqrt();
            let l1 = (-b - sqrt) / (2.0 * a);
            let l2 = (-b + sqrt) / (2.0 * a);
            let l = if l1 >= 0.0 {
                l1
            } else if l2 >= 0.0 {
                l2
            } else {
                return None;
            };

            let p = (pos + dir * l - self.pos) / self.radius;
            let mut angle = p.x.acos();
            if p.y < 0.0 {
                angle = 2.0 * PI - angle;
            }

            Some(ShapeHitInfo {
                length: l,
                x: angle / (2.0 * PI),
                side: 0,
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct Line {
    pub start: Vector2<f64>,
    pub end: Vector2<f64>,
}

impl Line {
    fn ray_cast(&self, pos: Vector2<f64>, dir: Vector2<f64>) -> Option<ShapeHitInfo> {
        let x1 = pos.x;
        let y1 = pos.y;
        let x2 = pos.x + dir.x;
        let y2 = pos.y + dir.y;

        let x3 = self.start.x;
        let y3 = self.start.y;
        let x4 = self.end.x;
        let y4 = self.end.y;

        let div = 1.0 / ((x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4));
        let t = div * ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4));
        let u = div * ((x1 - x3) * (y1 - y2) - (y1 - y3) * (x1 - x2));
        if t + 0.001 < 0.0 || u < 0.0 || u > 1.0 {
            None
        } else {
            let side = if (self.end - self.start).dot(dir) > 0.0 {
                0
            } else {
                1
            };
            Some(ShapeHitInfo {
                length: t,
                x: u,
                side,
            })
        }
    }
}
