use nalgebra::Matrix4;
use palette::{Pixel, Srgb};
use std::str::FromStr;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

use crate::geom::Dist;
use crate::plot::element_mesh::ElementMesh;
use crate::{
    compile_shader, link_program, upload_buffer_data, BYTES_PER_FLOAT,
    FLOATS_PER_POSITION, FLOATS_PER_VALUE, FLOATS_PER_VERTEX,
};

pub struct DensityLayer {
    program: WebGlProgram,

    u_color_map: WebGlUniformLocation,
    u_value_range: WebGlUniformLocation,
    u_transform: WebGlUniformLocation,

    vao: WebGlVertexArrayObject,
    array_buffer: WebGlBuffer,
    element_array_buffer: WebGlBuffer,
}

impl DensityLayer {
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
        let a_value = context.get_attrib_location(&program, "a_value") as u32;

        let u_color_map = context
            .get_uniform_location(&program, "u_color_map")
            .ok_or("Failed to get uniform location")?;
        let u_value_range = context
            .get_uniform_location(&program, "u_value_range")
            .ok_or("Failed to get uniform location")?;
        let u_transform = context
            .get_uniform_location(&program, "u_transform")
            .ok_or("Failed to get uniform location")?;

        // Create buffers
        let array_buffer =
            context.create_buffer().ok_or("Failed to create buffer")?;
        let element_array_buffer =
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
        context.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&element_array_buffer),
        );

        context.enable_vertex_attrib_array(a_position);
        context.vertex_attrib_pointer_with_i32(
            a_position,
            FLOATS_PER_POSITION,
            WebGl2RenderingContext::FLOAT,
            false,
            FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
            0,
        );

        context.enable_vertex_attrib_array(a_value);
        context.vertex_attrib_pointer_with_i32(
            a_value,
            FLOATS_PER_VALUE,
            WebGl2RenderingContext::FLOAT,
            false,
            FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
            FLOATS_PER_POSITION * BYTES_PER_FLOAT,
        );

        context.bind_vertex_array(None);

        Ok(Self {
            program,

            u_color_map,
            u_value_range,
            u_transform,

            vao,
            array_buffer,
            element_array_buffer,
        })
    }

    pub fn update_value_range(
        &self,
        context: &&WebGl2RenderingContext,
        range: [Dist; 2],
    ) {
        context.use_program(Some(&self.program));
        context.uniform2f(Some(&self.u_value_range), range[0], range[1]);
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

    fn update_color_map(&self, context: &WebGl2RenderingContext) {
        context.use_program(Some(&self.program));

        // TODO: Get gradient from CSS custom property, so it can change according to `prefers-color-scheme` media query
        // Colors generated using https://www.learnui.design/tools/gradient-generator.html
        let colors = "#1e2a4f, #053963, #004975, #005984, #006a8f, #007a94, #008a94, #009a8e, #00a984, #00b777, #51c467, #85cf57".split(", ").collect::<Vec<_>>();
        let colors_len = colors.len();
        let color_map_data = colors
            .into_iter()
            .enumerate()
            .flat_map(|(color_idx, hex)| {
                let w = color_idx as f32 / (colors_len - 1) as f32;
                let [r, g, b]: [f32; 3] =
                    Srgb::from_str(hex).unwrap().into_format().into_raw();
                [r, g, b, w]
            })
            .collect::<Vec<_>>();

        // Update color map
        context.uniform4fv_with_f32_array(
            Some(&self.u_color_map),
            &color_map_data,
        );
    }

    pub fn draw(
        &self,
        context: &WebGl2RenderingContext,
        mesh: &ElementMesh<Dist>,
    ) -> Result<(), String> {
        context.use_program(Some(&self.program));

        // Build vertex data
        let vertex_data = mesh.vertices();
        let index_data = &mesh
            .iter_triangle_elements()
            .flatten()
            .map(|idx| idx as u32)
            .collect();

        // Upload vertex data
        upload_buffer_data(
            context,
            &self.array_buffer,
            vertex_data,
            WebGl2RenderingContext::ARRAY_BUFFER,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        upload_buffer_data(
            context,
            &self.element_array_buffer,
            index_data,
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        self.update_color_map(context);

        // Draw the triangles
        context.bind_vertex_array(Some(&self.vao));
        context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            index_data.len() as i32,
            WebGl2RenderingContext::UNSIGNED_INT,
            0,
        );

        context.bind_vertex_array(None);

        Ok(())
    }
}
