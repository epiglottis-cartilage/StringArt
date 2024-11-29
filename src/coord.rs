#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord {
    pub x: f32,
    pub y: f32,
}

impl std::ops::Add for Coord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Coord {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl std::ops::Sub for Coord {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Coord {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl std::ops::Mul<f32> for Coord {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Coord {
            x: self.x * other,
            y: self.y * other,
        }
    }
}
impl std::ops::Div<f32> for Coord {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Coord {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
impl Coord {
    pub fn distance(self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    pub fn round(self) -> Self {
        Coord {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
}
