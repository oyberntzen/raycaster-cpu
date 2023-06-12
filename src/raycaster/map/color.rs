use cgmath::Vector2;

use image::io::Reader as ImageReader;
use image::DynamicImage;
use std::rc::Rc;

#[derive(Clone)]
pub enum Color {
    Solid([f64; 4]),
    Test,
    Test2,
    Texture(Rc<Texture>),
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
            Self::Texture(texture) => texture.sample(pos.x, pos.y),
        }
    }
}

pub struct Texture {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Texture {
    pub fn new(path: &str) -> Self {
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let width = img.width();
        let height = img.height();
        let data = if let DynamicImage::ImageRgba8(rgba8) = img {
            rgba8.into_raw()
        } else {
            let rgba8 = img.as_rgba8().unwrap();
            let mut pixels = Vec::<u8>::new();
            for pixel in rgba8.pixels() {
                for i in 0..4 {
                    pixels.push(pixel[i]);
                }
            }
            pixels
        };
        Self {
            width: width as usize,
            height: height as usize,
            data,
        }
    }

    pub fn sample(&self, x: f64, y: f64) -> [f64; 4] {
        let mut color = [0.0; 4];
        let xi = (x * (self.width as f64)) as usize;
        let yi = (y * (self.height as f64)) as usize;
        let index = (yi * self.width + xi) * 4;
        for i in 0..4 {
            color[i] = self.data[index + i] as f64 / 255.0;
        }
        color
    }
}
