use std::str::FromStr;

use glium::{implement_uniform_block, uniforms::{AsUniformValue, UniformValue}};
//use anyhow::Result;

#[derive(Clone, Copy)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

implement_uniform_block!(Colour, r, g, b, a);

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}, {}, {}, {}", self.r, self.g, self.b, self.a)
    }
}

impl FromStr for Colour {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = s
        .split(",")
        .map(|x| x.trim().parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()?;

        Ok(Self {
            r: val[0],
            g: val[1],
            b: val[2],
            a: val[3],
        })
    }
}

impl From<Colour> for [f32; 4] {
    fn from(value: Colour) -> Self {
        [value.r, value.g, value.b, value.a]
    }
}

impl AsUniformValue for Colour {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        UniformValue::Vec4(<[f32; 4]>::from(*self))
    }
}