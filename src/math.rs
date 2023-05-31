use std::ops::{Add, Mul, Sub};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn from_usize_tuple(tuple: (usize, usize)) -> Vector2 {
        Vector2 {
            x: tuple.0 as f32,
            y: tuple.1 as f32,
        }
    }

    pub fn set_x(&mut self, x: f32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn to_usize(&self) -> Result<(usize, usize), String> {
        let x = (self.x).floor() as i32;
        let y = (self.y).floor() as i32;

        if let (Some(x), Some(y)) = (x.try_into().ok(), y.try_into().ok()) {
            Ok((x, y))
        } else {
            Err("Vector2 cannot be converted to usize".to_string())
        }
    }

    pub fn to_tuple(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn interpolate(&self, other: &Vector2, t: f32) -> Vector2 {
        *self * (1.0 - t) + *other * t
    }

    pub fn normalize(&mut self) {
        let magnitude = self.magnitude();
        if magnitude <= 0.0 {
            panic!("Trying to normalize a zero length vector!");
        }
        self.x = self.x / magnitude;
        self.y = self.y / magnitude;
    }
}

impl Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, other: f32) -> Vector2 {
        Vector2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}
