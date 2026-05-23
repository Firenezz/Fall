use std::{fmt::Display, ops::{Deref, DerefMut}};

use bevy::reflect::Reflect;
use bevy::prelude::ReflectDefault;

use crate::units::temperature::*;

// Why fahrenheit is used to measure temperature is beyond me, but it is what it is.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Reflect)]
#[reflect(PartialEq, Default)]
pub struct Fahrenheit(pub(crate) f32);

impl Fahrenheit {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

impl From<f32> for Fahrenheit {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<Celsius> for Fahrenheit {
    fn from(value: Celsius) -> Self {
        Self::new(value.0 * 9.0 / 5.0 + 32.0)
    }
}

impl From<Kelvin> for Fahrenheit {
    fn from(value: Kelvin) -> Self {
        Self::new(value.0 * 9.0 / 5.0 - 459.67)
    }
}

impl AsRef<f32> for Fahrenheit {
    fn as_ref(&self) -> &f32 {
        &self.0
    }
}

impl AsMut<f32> for Fahrenheit {
    fn as_mut(&mut self) -> &mut f32 {
        &mut self.0
    }
}

impl Deref for Fahrenheit {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Fahrenheit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Fahrenheit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}°F", self.0)
    }
}

impl Default for Fahrenheit {
    fn default() -> Self {
        Self::new(32.0)
    }
}