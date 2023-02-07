use std::ops::Deref;

use crate::webgl::buffer::Buffer;
use crate::webgl::vertex::Vertex;

pub struct VertexBuffer<'a, T: Vertex> {
    buffer: Buffer<'a, T>,
}

impl<'a, T> Deref for VertexBuffer<'a, T>
where
    T: Vertex + Copy,
{
    type Target = Buffer<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<'a, T> From<Buffer<'a, T>> for VertexBuffer<'a, T>
where
    T: Vertex + Copy,
{
    fn from(buffer: Buffer<'a, T>) -> Self {
        Self { buffer }
    }
}
