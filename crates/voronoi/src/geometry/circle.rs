use bevy::math::Vec2;

use super::TestResult;


pub struct Circle {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,

    a_squared: f32,
    b_squared: f32,
    c_squared: f32,
}

impl Circle {
    pub fn new(a: Vec2, b: Vec2, c: Vec2) -> Self {
        Self { a, b, c,
            a_squared: a.x * a.x + a.y * a.y,
            b_squared: b.x * b.x + b.y * b.y,
            c_squared: c.x * c.x + c.y * c.y
        }
    }

    pub fn test(&self, point: Vec2) -> TestResult {
        // use determinant to check if the point is inside the circle

        let determinant = bevy::math::Mat4::from_cols(
            bevy::math::Vec4::new(self.a.x, self.b.x, self.c.x, point.x),
            bevy::math::Vec4::new(self.a.y, self.b.y, self.c.y, point.y),
            bevy::math::Vec4::new(self.a_squared, self.b_squared, self.c_squared, point.x * point.x + point.y * point.y),
            bevy::math::Vec4::new(1f32, 1f32, 1f32, 1f32)
        ).determinant();

        match determinant {
            x if x > 0f32 => TestResult::Inside,
            x if x == 0f32 => TestResult::Intersect,
            _ => TestResult::Outside
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inside() {
        let circle = Circle::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0));
        assert_eq!(circle.test(Vec2::new(0.5, 0.5)), TestResult::Inside);
        assert_eq!(circle.test(Vec2::new(1.0, 1.0)), TestResult::Intersect);
        assert_eq!(circle.test(Vec2::new(1.5, 0.5)), TestResult::Outside);
    }
}