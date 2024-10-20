use std::sync::Arc;

use glium::{
    index::NoIndices,
    uniforms::Uniforms,
    DrawError, DrawParameters, Frame, Program, Surface, VertexBuffer,
};

pub struct DrawItem<'a, T: Copy, U: Uniforms> {
    name: &'static str,
    buffer: VertexBuffer<T>,
    indices: NoIndices,
    program: Arc<Program>,
    uniforms: U,
    params: DrawParameters<'a>,
}

pub trait Drawable<'a, T: Copy, U: Uniforms> {
    fn upload(&mut self, data: &[T]);

    fn draw(&self, frame: &mut Frame) -> Result<(), DrawError>;
}

impl<'a, T: Copy, U: Uniforms> DrawItem<'a, T, U> {
    pub fn new(
        name: &'static str,
        buffer: VertexBuffer<T>,
        indices: NoIndices,
        program: Arc<Program>,
        uniforms: U,
    ) -> Self {
        Self {
            name,
            buffer,
            indices,
            program,
            uniforms,
            params: DrawParameters::default(),
        }
    }
}

impl<'a, T: Copy + std::fmt::Display, U: Uniforms> Drawable<'a, T, U> for DrawItem<'a, T, U> {
    fn upload(&mut self, data: &[T]) {
        
        if self.buffer.len() != data.len() {panic!("Error when drawing {}: Expected buffer size {}, got {}", self.name, self.buffer.len(), data.len())}
        self.buffer.write(data)
    }

    fn draw(&self, frame: &mut Frame) -> Result<(), DrawError> {
        frame.draw(
            &self.buffer,
            &self.indices,
            &self.program,
            &self.uniforms,
            &self.params,
        )
    }
}
