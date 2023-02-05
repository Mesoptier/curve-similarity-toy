use colorgrad::Gradient;
use itertools::Itertools;
use nalgebra::Matrix4;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlTexture,
    WebGlUniformLocation, WebGlVertexArrayObject,
};

use crate::geom::Dist;
use crate::plot::element_mesh::ElementMesh;
use crate::{
    compile_shader, link_program, upload_buffer_data, BYTES_PER_FLOAT,
    FLOATS_PER_POSITION, FLOATS_PER_VALUE, FLOATS_PER_VERTEX,
};

pub struct DensityLayer {
    program: WebGlProgram,

    u_value_range: WebGlUniformLocation,
    u_transform: WebGlUniformLocation,
    u_gradient: WebGlUniformLocation,
    gradient_texture: WebGlTexture,

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

        let u_value_range = context
            .get_uniform_location(&program, "u_value_range")
            .ok_or("Failed to get uniform location")?;
        let u_transform = context
            .get_uniform_location(&program, "u_transform")
            .ok_or("Failed to get uniform location")?;

        let u_gradient = context
            .get_uniform_location(&program, "u_gradient")
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

        // Create gradient textures
        let gradient_texture =
            context.create_texture().ok_or("Failed to create texture")?;

        Ok(Self {
            program,

            u_value_range,
            u_transform,
            u_gradient,
            gradient_texture,

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

    pub fn update_gradient_smooth(
        &self,
        context: &WebGl2RenderingContext,
        gradient: Gradient,
    ) -> Result<(), String> {
        self.set_gradient_texture_filter(
            context,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.update_gradient(context, gradient, 256)
    }

    pub fn update_gradient_sharp(
        &self,
        context: &WebGl2RenderingContext,
        gradient: Gradient,
        segments: usize,
    ) -> Result<(), String> {
        self.set_gradient_texture_filter(
            context,
            WebGl2RenderingContext::NEAREST as i32,
        );
        self.update_gradient(context, gradient.sharp(segments, 0.), segments)
    }

    fn set_gradient_texture_filter(
        &self,
        context: &WebGl2RenderingContext,
        param: i32,
    ) {
        context.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.gradient_texture),
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            param,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            param,
        );
    }

    fn update_gradient(
        &self,
        context: &WebGl2RenderingContext,
        gradient: Gradient,
        size: usize,
    ) -> Result<(), String> {
        let pixels = gradient
            .colors(size)
            .into_iter()
            .flat_map(|color| color.to_rgba8())
            .collect_vec();

        context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            size as i32,
            1,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&pixels)
        ).map_err(|err| format!("{err:?}"))?;

        Ok(())
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

        // Bind gradient texture
        context.active_texture(WebGl2RenderingContext::TEXTURE0);
        context.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.gradient_texture),
        );
        context.uniform1i(Some(&self.u_gradient), 0);

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
