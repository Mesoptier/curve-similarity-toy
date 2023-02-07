use nalgebra::Matrix4;
use web_sys::{
    WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

use crate::geom::Dist;
use crate::plot::element_mesh::Vertex;
use crate::webgl::buffer::{Buffer, BufferTarget, BufferUsage};
use crate::webgl::vertex_buffer::VertexBuffer;
use crate::{
    compile_shader, link_program, BYTES_PER_FLOAT, FLOATS_PER_POSITION,
    FLOATS_PER_VERTEX,
};

pub struct ContourLinesLayer<'a> {
    program: WebGlProgram,
    u_transform: WebGlUniformLocation,
    vao: WebGlVertexArrayObject,
    vertex_buffer: VertexBuffer<'a, Vertex<Dist>>,
}

impl<'a> ContourLinesLayer<'a> {
    pub fn new(context: &'a WebGl2RenderingContext) -> Result<Self, String> {
        // Compiler shaders
        let vert_shader = compile_shader(
            context,
            WebGl2RenderingContext::VERTEX_SHADER,
            include_str!("shader.vert"),
        )?;
        let frag_shader = compile_shader(
            context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            include_str!("shader.frag"),
        )?;

        // Create & link program
        let program = link_program(context, &vert_shader, &frag_shader)?;

        // Get attributes and uniforms
        let a_position =
            context.get_attrib_location(&program, "a_position") as u32;

        let u_transform = context
            .get_uniform_location(&program, "u_transform")
            .ok_or("Failed to get uniform location")?;

        // Create buffers
        let vertex_buffer: VertexBuffer<Vertex<Dist>> = Buffer::new(
            context,
            BufferTarget::ArrayBuffer,
            BufferUsage::StaticDraw,
        )
        .map_err(|error| format!("{error:?}"))?
        .into();

        // Setup vertex array object
        let vao = context
            .create_vertex_array()
            .ok_or("Failed to create vertex array object")?;

        context.bind_vertex_array(Some(&vao));

        vertex_buffer.bind();

        context.enable_vertex_attrib_array(a_position);
        context.vertex_attrib_pointer_with_i32(
            a_position,
            FLOATS_PER_POSITION,
            WebGl2RenderingContext::FLOAT,
            false,
            // TODO: Pass positions instead of vertices?
            FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
            0,
        );

        context.bind_vertex_array(None);

        Ok(Self {
            program,
            u_transform,
            vao,
            vertex_buffer,
        })
    }

    pub fn update_transform(
        &self,
        context: &WebGl2RenderingContext,
        mat: Matrix4<Dist>,
    ) {
        context.use_program(Some(&self.program));
        context.uniform_matrix4fv_with_f32_array(
            Some(&self.u_transform),
            false,
            mat.transpose().data.as_slice(),
        );
    }

    pub fn draw(
        &self,
        context: &WebGl2RenderingContext,
        vertex_data: Vec<Vertex<Dist>>,
    ) -> Result<(), String> {
        context.use_program(Some(&self.program));

        self.vertex_buffer.write(&vertex_data);

        context.bind_vertex_array(Some(&self.vao));
        context.draw_arrays(
            WebGl2RenderingContext::LINES,
            0,
            vertex_data.len() as i32,
        );

        context.bind_vertex_array(None);

        Ok(())
    }
}
