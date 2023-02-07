use crate::webgl::buffer::Buffer;
use std::ops::Deref;
use bytemuck::Pod;

pub struct IndexBuffer<'a, T: Pod> {
    buffer: Buffer<'a, T>,
}

impl<'a, T> Deref for IndexBuffer<'a, T>
where
    T: Pod + Copy,
{
    type Target = Buffer<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<'a, T> From<Buffer<'a, T>> for IndexBuffer<'a, T>
where
    T: Pod + Copy,
{
    fn from(buffer: Buffer<'a, T>) -> Self {
        Self { buffer }
    }
}
