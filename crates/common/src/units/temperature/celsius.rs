use std::{fmt::Display, ops::{Deref, DerefMut}};

use crate::units::temperature::*;
use bevy::reflect::Reflect;
use bevy::prelude::ReflectDefault;


#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Reflect)]
#[reflect(PartialEq, Default)]
pub struct Celsius(pub(crate) f32);

impl Celsius {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

impl From<Kelvin> for Celsius {
    fn from(value: Kelvin) -> Self {
        Self::new(value.0 - 273.15)
    }
}

impl From<Fahrenheit> for Celsius {
    fn from(value: Fahrenheit) -> Self {
        Self::new((value.0 - 32.0) * 5.0 / 9.0)
    }
}

impl From<f32> for Celsius {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl AsRef<f32> for Celsius {
    fn as_ref(&self) -> &f32 {
        &self.0
    }
}

impl AsMut<f32> for Celsius {
    fn as_mut(&mut self) -> &mut f32 {
        &mut self.0
    }
}

impl Deref for Celsius {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Celsius {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Celsius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}°C", self.0)
    }
}

impl Default for Celsius {
    fn default() -> Self {
        Self::new(0.0)
    }
}