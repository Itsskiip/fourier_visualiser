use std::sync::Arc;

use anyhow::anyhow;
use glium::{
    glutin::surface::WindowSurface,
    implement_vertex,
    index::{NoIndices, PrimitiveType::LineStrip},
    uniforms::{EmptyUniforms, UniformsStorage},
    Program, VertexBuffer,
};

use num::Complex;

use crate::{
    graphics::{DrawItem, Drawable},
    Colour, Complex32,
};

#[derive(Copy, Clone, Default)]
pub struct BarVertex {
    pub position: [f32; 2],
    pub alpha: f32,
}

implement_vertex!(BarVertex, position);

impl std::fmt::Display for BarVertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.position[0], self.position[1])
    }
}

impl From<BarVertex> for Complex32 {
    fn from(value: BarVertex) -> Self {
        Complex {
            re: value.position[0],
            im: value.position[1],
        }
    }
}

impl TryFrom<&str> for BarVertex {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut v = value.split(",").map(|x| anyhow::Ok(x.trim().parse()?));
        let err = "Unable to parse value as a vertex";
        Ok(Self {
            position: [
                v.next().ok_or(anyhow!(err))??,
                v.next().ok_or(anyhow!(err))??,
            ],
            alpha: 1.0,
        })
    }
}

impl From<Complex32> for BarVertex {
    fn from(value: Complex32) -> Self {
        BarVertex {
            position: [value.re, value.im],
            alpha: Default::default(),
        }
    }
}

pub type BarUniform<'a> = UniformsStorage<'a, Colour, EmptyUniforms>;

pub struct Bar<'a> {
    draw_item: DrawItem<'a, BarVertex, BarUniform<'a>>,
}

impl<'a> Bar<'a> {
    pub fn new(
        facade: &glium::Display<WindowSurface>,
        samples: usize,
        program: Arc<Program>,
        colour: Colour,
    ) -> Self {
        Self {
            draw_item: DrawItem::new(
                "Bar Vertex",
                VertexBuffer::empty_dynamic(facade, samples).unwrap(),
                NoIndices(LineStrip),
                program,
                uniform! {vertex_colour: colour},
            ),
        }
    }
}

impl<'a> Drawable<'a, BarVertex, BarUniform<'a>> for Bar<'a> {
    fn upload(&mut self, data: &[BarVertex]) {
        self.draw_item.upload(data);
    }

    fn draw(&self, frame: &mut glium::Frame) -> Result<(), glium::DrawError> {
        self.draw_item.draw(frame)
    }
}

pub fn get_program(facade: &glium::Display<WindowSurface>) -> glium::Program {
    let vertex_shader_src = r#"
            #version 140

            in vec2 position;
            in float alpha;

            out float v_alpha;

            void main() {
                vec2 pos = position;
                gl_Position = vec4(pos, 0.0, 1.0);
            }
        "#;

    let fragment_shader_src = r#"
            #version 140

            uniform vec4 vertex_colour;
            in float v_alpha;
            out vec4 color;

            void main() {
                color = vec4(vertex_colour.r, vertex_colour.g, vertex_colour.b, v_alpha);
            }
        "#;

    glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None).unwrap()
}
