use crate::geom::Dist;
use crate::plot::element_mesh::Vertex;
use crate::{
    compile_shader, link_program, upload_buffer_data, BYTES_PER_FLOAT,
    FLOATS_PER_POSITION, FLOATS_PER_VERTEX,
};
use nalgebra::Matrix4;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

pub struct ContourLinesLayer {
    program: WebGlProgram,
    u_transform: WebGlUniformLocation,
    vao: WebGlVertexArrayObject,
    array_buffer: WebGlBuffer,
}

impl ContourLinesLayer {
    pub fn new(context: &WebGl2RenderingContext) -> Result<Self, String> {
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
        let array_buffer =
            context.create_buffer().ok_or("Failed to create buffer")?;

        // Setup vertex array object
        let vao = context
            .create_vertex_array()
            .ok_or("Failed to create vertex array object")?;

        context.bind_vertex_array(Some(&vao));

        context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&array_buffer),
        );

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
            array_buffer,
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

        upload_buffer_data(
            context,
            &self.array_buffer,
            &vertex_data,
            WebGl2RenderingContext::ARRAY_BUFFER,
            WebGl2RenderingContext::STATIC_DRAW,
        );

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
