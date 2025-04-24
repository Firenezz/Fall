use std::ops::{Add, Div, Mul, Neg, Sub};



pub trait Coord: Send + Sync + Copy {
    type Inner: Send + Sync + Copy + Sub<Output = Self::Inner> + Add<Output = Self::Inner> + Mul<Output = Self::Inner> + Div<Output = Self::Inner> + Neg<Output = Self::Inner> + PartialEq;
    fn new(x: Self::Inner, y: Self::Inner) -> Self;
    fn x(&self) -> Self::Inner;
    fn y(&self) -> Self::Inner;

    /// Return the magnitude of the 2D vector represented by (x, y)
    fn magnitude2(&self) -> Self::Inner {
        self.x() * self.x() + self.y() * self.y()
    }
}

impl Coord for bevy::math::Vec2 {
    type Inner = f32;
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    #[inline]
    fn x(&self) -> f32 {
        self.x
    }
    #[inline]
    fn y(&self) -> f32 {
        self.y  
    }
}

pub trait Vector: Send + Sync + Copy + Coord {
    fn from_coord(a: Self, b: Self) -> Self {
        Self::new ( a.x() - b.x(), a.y() - b.y())
    }

    #[inline]
    fn dot(&self, other: Self) -> Self::Inner {
        self.x() * other.x() + self.y() * other.y()
    }

    #[inline]
    fn cross(&self, other: Self) -> Self::Inner {
        self.x() * other.y() - self.y() * other.x()
    }
}

impl Vector for bevy::math::Vec2 {}
impl Vector for bevy::math::IVec2 {}

impl Coord for bevy::math::IVec2 {
    type Inner = i32;
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    #[inline]
    fn x(&self) -> i32 {
        self.x
    }
    #[inline]
    fn y(&self) -> i32 {
        self.y  
    }
}

impl Coord for (f32, f32) {
    type Inner = f32;
    fn new(x: f32, y: f32) -> Self {
        (x, y)
    }
    #[inline]
    fn x(&self) -> f32 {
        self.0
    }
    #[inline]
    fn y(&self) -> f32 {
        self.1
    }
}