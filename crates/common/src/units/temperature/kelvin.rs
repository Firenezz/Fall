use std::{
    fmt::Display,
    ops::{Deref, DerefMut, Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign}
};

use crate::units::temperature::*;

use bevy::reflect::Reflect;
use bevy::prelude::ReflectDefault;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Reflect)]
#[reflect(PartialEq, Default)]
pub struct Kelvin(pub(crate) TemperatureValue);

impl Kelvin {
    pub fn new(value: TemperatureValue) -> Self {
        Self(value)
    }
}

impl From<Celsius> for Kelvin {
    fn from(value: Celsius) -> Self {
        Self::new(value.0 + 273.15)
    }
}

impl From<Fahrenheit> for Kelvin {
    fn from(value: Fahrenheit) -> Self {
        Self::new(value.0 * 9.0 / 5.0 + 273.15)
    }
}

impl From<f32> for Kelvin {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl AsRef<TemperatureValue> for Kelvin {
    fn as_ref(&self) -> &TemperatureValue {
        &self.0
    }
}

impl AsMut<TemperatureValue> for Kelvin {
    fn as_mut(&mut self) -> &mut TemperatureValue {
        &mut self.0
    }
}

impl Deref for Kelvin {
    type Target = TemperatureValue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Kelvin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Kelvin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}K", self.0)
    }
}

impl Default for Kelvin {
    /// 0°C = 273.15K
    fn default() -> Self {
        Self::new(273.15)
    }
}

// Operations with &Kelvin and TemperatureValue

impl Add<TemperatureValue> for &Kelvin {
    type Output = Kelvin;
    fn add(self, other: TemperatureValue) -> Self::Output {
        Kelvin::new(self.0 + other)
    }
}

impl Sub<TemperatureValue> for &Kelvin {
    type Output = Kelvin;
    fn sub(self, other: TemperatureValue) -> Self::Output {
        Kelvin::new(self.0 - other)
    }
}

impl Mul<TemperatureValue> for &Kelvin {
    type Output = Kelvin;
    fn mul(self, other: TemperatureValue) -> Self::Output {
        Kelvin::new(self.0 * other)
    }
}

impl Div<TemperatureValue> for &Kelvin {
    type Output = Kelvin;
    fn div(self, other: TemperatureValue) -> Self::Output {
        Kelvin::new(self.0 / other)
    }
}

// Operations with owned Kelvin and TemperatureValue

impl Add<TemperatureValue> for Kelvin {
    type Output = Kelvin;
    fn add(self, other: TemperatureValue) -> Self::Output {
        Kelvin::new(self.0 + other)
    }
}

impl Sub<TemperatureValue> for Kelvin {
    type Output = Kelvin;
    fn sub(self, other: TemperatureValue) -> Self::Output {
        Kelvin::new(self.0 - other)
    }
}

impl Mul<TemperatureValue> for Kelvin {
    type Output = Kelvin;
    fn mul(self, other: TemperatureValue) -> Self::Output {
        Kelvin::new(self.0 * other)
    }
}

impl Div<TemperatureValue> for Kelvin {
    type Output = Kelvin;
    fn div(self, other: TemperatureValue) -> Self::Output {
        Kelvin::new(self.0 / other)
    }
}

// Operations with &Kelvin and Kelvin

impl Add<Kelvin> for &Kelvin {
    type Output = f32;
    fn add(self, other: Kelvin) -> Self::Output {
        self.0 + other.0
    }
}

impl Sub<Kelvin> for &Kelvin {
    type Output = f32;
    fn sub(self, other: Kelvin) -> Self::Output {
        self.0 - other.0
    }
}

impl Add<Kelvin> for Kelvin {
    type Output = Kelvin;
    fn add(self, other: Kelvin) -> Self::Output {
        Kelvin::new(self.0 + other.0)
    }
}

impl Sub<Kelvin> for Kelvin {
    type Output = Kelvin;
    fn sub(self, other: Kelvin) -> Self::Output {
        Kelvin::new(self.0 - other.0)
    }
}

// Assignments

impl AddAssign<TemperatureValue> for Kelvin {
    fn add_assign(&mut self, other: TemperatureValue) {
        *self = *self + other;
    }
}

impl SubAssign<TemperatureValue> for Kelvin {
    fn sub_assign(&mut self, other: TemperatureValue) {
        *self = *self - other;
    }
}

impl MulAssign<TemperatureValue> for Kelvin {
    fn mul_assign(&mut self, other: TemperatureValue) {
        *self = *self * other;
    }
}

impl DivAssign<TemperatureValue> for Kelvin {
    fn div_assign(&mut self, other: TemperatureValue) {
        *self = *self / other;
    }
}

impl AddAssign<Kelvin> for &Kelvin {
    fn add_assign(&mut self, other: Kelvin) {
        *self = *self + other;
    }
}

impl SubAssign<Kelvin> for &mut Kelvin {
    fn sub_assign(&mut self, other: Kelvin) {
        *self = *self - other;
    }
}

impl MulAssign<Kelvin> for &mut Kelvin {
    fn mul_assign(&mut self, other: Kelvin) {
        *self = *self * other;
    }
}

impl DivAssign<Kelvin> for &mut Kelvin {
    fn div_assign(&mut self, other: Kelvin) {
        *self = *self / other;
    }
}