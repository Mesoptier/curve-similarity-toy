use std::marker::PhantomData;

use bytemuck::{cast_slice, Pod};
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::webgl::error::Error;

macro_rules! gl_enum {
    ($enum_name:ident, { $($value_name:ident => $gl_value_name:ident),+ }) => {
        pub enum $enum_name {
            $($value_name),+
        }

        impl $enum_name {
            fn gl_enum(&self) -> u32 {
                match (self) {
                    $($enum_name::$value_name => WebGl2RenderingContext::$gl_value_name),+
                }
            }
        }
    };
}

gl_enum!(BufferTarget, {
    ArrayBuffer => ARRAY_BUFFER,
    ElementArrayBuffer => ELEMENT_ARRAY_BUFFER,
    CopyReadBuffer => COPY_READ_BUFFER,
    CopyWriteBuffer => COPY_WRITE_BUFFER,
    TransformFeedbackBuffer => TRANSFORM_FEEDBACK_BUFFER,
    UniformBuffer => UNIFORM_BUFFER,
    PixelPackBuffer => PIXEL_PACK_BUFFER,
    PixelUnpackBuffer => PIXEL_UNPACK_BUFFER
});

gl_enum!(BufferUsage, {
    StaticDraw => STATIC_DRAW,
    DynamicDraw => DYNAMIC_DRAW,
    StreamDraw => STREAM_DRAW,
    StaticRead => STATIC_READ,
    DynamicRead => DYNAMIC_READ,
    StreamRead => STREAM_READ,
    StaticCopy => STATIC_COPY,
    DynamicCopy => DYNAMIC_COPY,
    StreamCopy => STREAM_COPY
});

pub struct Buffer<'a, T: Pod> {
    context: &'a WebGl2RenderingContext,
    buffer: WebGlBuffer,
    target: BufferTarget,
    usage: BufferUsage,

    // TODO: Should this be `PhantomData<&'a T>`?
    data_type: PhantomData<T>,
}

impl<'a, T: Pod> Buffer<'a, T> {
    pub fn new(
        context: &'a WebGl2RenderingContext,
        target: BufferTarget,
        usage: BufferUsage,
    ) -> Result<Self, Error> {
        let buffer = context.create_buffer().ok_or(Error::CreateBuffer)?;

        Ok(Self {
            context,
            buffer,
            target,
            usage,
            data_type: PhantomData,
        })
    }

    // TODO: This function should not be exposed
    pub fn bind(&self) {
        self.context
            .bind_buffer(self.target.gl_enum(), Some(&self.buffer));
    }

    pub fn write(&self, data: &[T]) {
        self.bind();

        let bytes: &[u8] = cast_slice(data);
        // TODO: Use buffer_sub_data?
        self.context.buffer_data_with_u8_array(
            self.target.gl_enum(),
            bytes,
            self.usage.gl_enum(),
        );
    }
}
