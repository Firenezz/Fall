use std::{
    fmt::Display,
    ops::{Deref, DerefMut}
};

use crate::units::temperature::*;

use bevy::reflect::Reflect;
use bevy::prelude::ReflectDefault;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Reflect)]
#[reflect(PartialEq, Default)]
pub struct Kelvin(pub(crate) TemperatureValue);

impl Kelvin {
    /// Creates a new Kelvin value, clamping negative values to 0.0
    /// Use this for absolute temperatures (which cannot be negative)
    pub fn new(value: TemperatureValue) -> Self {
        if value.is_sign_negative() {
            bevy::log::error!("Temperature value is negative: {}", value);
            return Self(0.0);
        }
        Self(value)
    }

    /// Creates a new Kelvin value without validation
    /// Use this for temperature differences or internal arithmetic operations
    /// where negative values are valid (e.g., temperature differences)
    pub(crate) fn new_unchecked(value: TemperatureValue) -> Self {
        Self(value)
    }

    pub fn get_value(&self) -> TemperatureValue {
        self.0
    }

    pub fn set_value(&mut self, value: TemperatureValue) {
        if value.is_sign_negative() {
            bevy::log::error!("Temperature value is negative: {}", value);
            return;
        }
        self.0 = value;
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

auto_ops::impl_op_ex_commutative!(+ |left: &Kelvin, right: &TemperatureValue| -> Kelvin {
    Kelvin::new_unchecked(left.0 + right)
});

auto_ops::impl_op_ex!(- |left: &Kelvin, right: &TemperatureValue| -> Kelvin {
    Kelvin::new_unchecked(left.0 - right)
});

auto_ops::impl_op_ex_commutative!(* |left: &Kelvin, right: &TemperatureValue| -> Kelvin {
    Kelvin::new_unchecked(left.0 * right)
});

auto_ops::impl_op_ex!(/ |left: &Kelvin, right: &TemperatureValue| -> Kelvin {
    Kelvin::new_unchecked(left.0 / right)
});

// Operations with Kelvin and Kelvin

auto_ops::impl_op_ex!(+ |left: &Kelvin, right: &Kelvin| -> Kelvin {
    Kelvin::new_unchecked(left.0 + right.0)
});

auto_ops::impl_op_ex!(- |left: &Kelvin, right: &Kelvin| -> Kelvin {
    Kelvin::new_unchecked(left.0 - right.0)
});

auto_ops::impl_op_ex!(* |left: &Kelvin, right: &Kelvin| -> Kelvin {
    Kelvin::new_unchecked(left.0 * right.0)
});

auto_ops::impl_op_ex!(/ |left: &Kelvin, right: &Kelvin| -> Kelvin {
    Kelvin::new_unchecked(left.0 / right.0)
});

// Assignments

auto_ops::impl_op_ex!(+= |left: &mut Kelvin, right: &TemperatureValue| {
    left.0 += *right;
});

auto_ops::impl_op_ex!(-= |left: &mut Kelvin, right: &TemperatureValue| {
    left.0 -= *right
});

auto_ops::impl_op_ex!(*= |left: &mut Kelvin, right: &TemperatureValue| {
    left.0 *= *right
});

auto_ops::impl_op_ex!(/= |left: &mut Kelvin, right: &TemperatureValue| {
    left.0 /= *right
});