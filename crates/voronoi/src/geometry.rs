
pub mod circle;
pub mod edge;
pub mod locations;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestResult {
    Outside,
    Inside,
    Intersect
}

pub trait Abs {
    fn abs(&self) -> Self;
}

impl Abs for i32 {
    fn abs(&self) -> Self {
        match self.is_positive() {
            true => *self,
            false => -self
        }
    }
}

impl Abs for i64 {
    fn abs(&self) -> Self {
        match self.is_positive() {
            true => *self,
            false => -self
        }
    }
}

impl Abs for f32 {
    fn abs(&self) -> Self {
        match self {
            x if x.is_infinite() || x.is_nan() => *x,
            _ => match self.is_sign_positive() {
                true => *self,
                false => -self
            }
        }
    }
}

impl Abs for f64 {
    fn abs(&self) -> Self {
        match self {
            x if x.is_infinite() || x.is_nan() => *x,
            _ => match self.is_sign_positive() {
                true => *self,
                false => -self
            }
        }
    }
}