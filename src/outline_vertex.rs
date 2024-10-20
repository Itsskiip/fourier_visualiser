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
pub struct OutlineVertex {
    pub position: [f32; 2],
    pub alpha: f32,
}

implement_vertex!(OutlineVertex, position);

impl std::fmt::Display for OutlineVertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.position[0], self.position[1])
    }
}

impl From<OutlineVertex> for Complex32 {
    fn from(value: OutlineVertex) -> Self {
        Complex {
            re: value.position[0],
            im: value.position[1],
        }
    }
}

impl TryFrom<&str> for OutlineVertex {
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

impl From<Complex32> for OutlineVertex {
    fn from(value: Complex32) -> Self {
        OutlineVertex {
            position: [value.re, value.im],
            alpha: Default::default(),
        }
    }
}

pub type OutlineUniform<'a> = UniformsStorage<'a, Colour, EmptyUniforms>;

pub struct Outline<'a> {
    draw_item: DrawItem<'a, OutlineVertex, OutlineUniform<'a>>,
}

impl<'a> Outline<'a> {
    pub fn new(
        facade: &glium::Display<WindowSurface>,
        samples: usize,
        program: Arc<Program>,
        colour: Colour,
    ) -> Self {
        Self {
            draw_item: DrawItem::new(
                "Outline Vertex",
                VertexBuffer::empty_dynamic(facade, samples).unwrap(),
                NoIndices(LineStrip),
                program,
                uniform! {vertex_colour: colour},
            ),
        }
    }
}

impl<'a> Drawable<'a, OutlineVertex, OutlineUniform<'a>> for Outline<'a> {
    fn upload(&mut self, data: &[OutlineVertex]) {
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
