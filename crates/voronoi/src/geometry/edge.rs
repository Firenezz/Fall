use bevy::math::{FloatPow, Vec2};

use super::locations::{Coord, Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge<Data> {
    pub a: Data,
    pub b: Data,
    pub c: Data
}

impl Edge<f32> {
    pub fn new<C: Coord<Inner = f32>>(a: C::Inner, b: C::Inner, c: C::Inner) -> Self {
        Self { a, b, c }
    }

    pub fn from_points<C: Coord<Inner = f32> + Vector>(point_a: C, point_b: C) -> Self {
        let delta = <C as Vector>::from_coord(point_b, point_a);

        let abs_delta_x = delta.x().abs();
        let abs_delta_y = delta.y().abs();

        let bisecting_constant = 
            point_a.x() * delta.x() + point_a.y() * delta.y() + 
            ( 0.5f32 * (delta.x().squared() + delta.y().squared()));

        if abs_delta_x < abs_delta_y {
            Self { 
                a: 1f32, 
                b: delta.x() / delta.y(),
                c: bisecting_constant / delta.y()
            }
        } else {
            Self { 
                a: 1f32, 
                b: delta.y() / delta.x(),
                c: bisecting_constant / delta.x()
            }
        }
    }

    pub fn evaluate(&self, x: f32, y: f32) -> bool {
        match self.a * x + self.b * y + self.c {
            0f32 => true,
            _ => false
        }
    }

    pub fn evaluate_y(&self, x: f32) -> f32 {
        -(self.a * x - self.c) / self.b
    }

    pub fn evaluate_x(&self, y: f32) -> f32 {
        -(self.b * y - self.c) / self.a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_method() {
        let point_a = Vec2::new(-2.0, 2.0);
        let point_b = Vec2::new(1.0, -1.0);
        let edge = Edge::<f32>::from_points(
            point_a,
            point_b
        );

        assert_eq!(edge, Edge { a: 1f32, b: -1f32, c: -1f32 });
        assert_eq!(edge.evaluate_y(0.0), 1.0);
        assert_eq!(edge.evaluate_y(1.0), 2.0);
        assert_eq!(edge.evaluate_y(1.5), 2.5);
        assert_eq!(edge.evaluate_y(2.0), 3.0);
    }

    #[test]
    fn test_evaluation_y_method() {
        let point_a = Vec2::new(-2.0, 2.0);
        let point_b = Vec2::new(1.0, -1.0);
        let edge = Edge::<f32>::from_points(
            point_a,
            point_b
        );

        assert_eq!(edge, Edge { a: 1f32, b: -1f32, c: -1f32 });
        assert_eq!(edge.evaluate_y(0.0), 1.0);
        assert_eq!(edge.evaluate_y(1.0), 2.0);
        assert_eq!(edge.evaluate_y(1.5), 2.5);
        assert_eq!(edge.evaluate_y(2.0), 3.0);
    }

    #[test]
    fn test_evaluation_x_method() {
        let point_a = Vec2::new(-2.0, 2.0);
        let point_b = Vec2::new(1.0, -1.0);
        let edge = Edge::<f32>::from_points(
            point_a,
            point_b
        );

        assert_eq!(edge, Edge { a: 1f32, b: -1f32, c: -1f32 });
        assert_eq!(edge.evaluate_x(0.0), -1.0);
        assert_eq!(edge.evaluate_x(1.0), 0.0);
        assert_eq!(edge.evaluate_x(1.5), 0.5);
        assert_eq!(edge.evaluate_x(2.0), 1.0);
    }
}