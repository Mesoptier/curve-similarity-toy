use std::iter;
use std::str::FromStr;

use itertools::Itertools;
use palette::{Pixel, Srgb};
use wasm_bindgen::prelude::*;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader,
    WebGlUniformLocation, WebGlVertexArrayObject,
};

use crate::geom::curve::Curve;
use crate::geom::{Dist, JsCurve};
use crate::plot::curve_dist_fn::CurveDistFn;
use crate::plot::element_mesh::{ElementMesh, Vertex};
use crate::plot::isolines;
use crate::plot::isolines::BuildIsolines;
use crate::traits::mix::Mix;
use crate::traits::vec_ext::VecExt;

mod geom;
mod plot;
mod traits;

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format!($($t)*).into()))
}

const BYTES_PER_FLOAT: i32 = 4;

const FLOATS_PER_POSITION: i32 = 2;
const FLOATS_PER_VALUE: i32 = 1;
const FLOATS_PER_VERTEX: i32 = FLOATS_PER_POSITION + FLOATS_PER_VALUE;

fn subdivide_lengths(lengths: &Vec<Dist>, res: Dist) -> Vec<Dist> {
    if lengths.is_empty() {
        return vec![];
    }

    iter::once(*lengths.first().unwrap())
        .chain(
            lengths
                .iter()
                .tuple_windows::<(_, _)>()
                .flat_map(|(l1, l2)| {
                    // Note: since `num_subdivisions` is 0 if both lengths are equal, this
                    // effectively also deduplicates the lengths
                    let num_subdivisions = ((l2 - l1) / res).ceil() as usize;
                    (0..num_subdivisions).map(move |i| {
                        let t = (i + 1) as Dist / (num_subdivisions as Dist);
                        l1 * (1. - t) + l2 * t
                    })
                }),
        )
        .collect()
}

#[wasm_bindgen(getter_with_clone)]
pub struct Plotter {
    context: WebGl2RenderingContext,
    curves: [Curve; 2],

    color_map_uniform: WebGlUniformLocation,
    value_range_uniform: WebGlUniformLocation,
    transform_uniform: WebGlUniformLocation,

    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    isoline_vertex_buffer: WebGlBuffer,

    vao_triangles: WebGlVertexArrayObject,
    vao_isolines: WebGlVertexArrayObject,
}

#[wasm_bindgen]
impl Plotter {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &WebGl2RenderingContext) -> Result<Plotter, JsValue> {
        let context = context.clone();

        // Init: compiler shaders
        let vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            include_str!("shader.vert"),
        )?;
        let frag_shader = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            include_str!("shader.frag"),
        )?;

        // Init: create & link program
        let program = link_program(&context, &vert_shader, &frag_shader)?;
        context.use_program(Some(&program));

        // Init: get attributes and uniforms
        let position_attribute =
            context.get_attrib_location(&program, "a_position") as u32;
        let value_attribute =
            context.get_attrib_location(&program, "a_value") as u32;

        let color_map_uniform = context
            .get_uniform_location(&program, "u_color_map")
            .unwrap();
        let value_range_uniform = context
            .get_uniform_location(&program, "u_value_range")
            .unwrap();
        let transform_uniform = context
            .get_uniform_location(&program, "u_transform")
            .unwrap();

        // Init: create buffers
        let vertex_buffer =
            context.create_buffer().ok_or("Failed to create buffer")?;
        let index_buffer =
            context.create_buffer().ok_or("Failed to create buffer")?;
        let isoline_vertex_buffer =
            context.create_buffer().ok_or("Failed to create buffer")?;

        // Init: setup VertexArrayObject for main triangle mesh
        let vao_triangles = context
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        context.bind_vertex_array(Some(&vao_triangles));

        context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&vertex_buffer),
        );
        context.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&index_buffer),
        );

        context.enable_vertex_attrib_array(position_attribute);
        context.vertex_attrib_pointer_with_i32(
            position_attribute,
            FLOATS_PER_POSITION,
            WebGl2RenderingContext::FLOAT,
            false,
            FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
            0,
        );

        context.enable_vertex_attrib_array(value_attribute);
        context.vertex_attrib_pointer_with_i32(
            value_attribute,
            FLOATS_PER_VALUE,
            WebGl2RenderingContext::FLOAT,
            false,
            FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
            FLOATS_PER_POSITION * BYTES_PER_FLOAT,
        );

        context.bind_vertex_array(None);

        // Init: setup VertexArrayObject for isolines
        let vao_isolines = context
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        context.bind_vertex_array(Some(&vao_isolines));

        context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&isoline_vertex_buffer),
        );

        context.enable_vertex_attrib_array(position_attribute);
        context.vertex_attrib_pointer_with_i32(
            position_attribute,
            FLOATS_PER_POSITION,
            WebGl2RenderingContext::FLOAT,
            false,
            FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
            0,
        );

        // TODO: Use alternative vertex shader with a color uniform instead of a value attribute
        context.vertex_attrib1f(value_attribute, -1.0);

        context.bind_vertex_array(None);

        // Enable blending
        context.enable(WebGl2RenderingContext::BLEND);
        context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        Ok(Self {
            context,
            curves: [Curve::default(), Curve::default()],

            color_map_uniform,
            value_range_uniform,
            transform_uniform,

            vertex_buffer,
            index_buffer,
            isoline_vertex_buffer,

            vao_triangles,
            vao_isolines,
        })
    }

    pub fn draw(&self, show_mesh: bool) {
        let context = &self.context;

        // Build mesh
        let res = 64.;
        let x_points =
            subdivide_lengths(self.curves[0].cumulative_lengths(), res);
        let y_points =
            subdivide_lengths(self.curves[1].cumulative_lengths(), res);

        let mut min_v = f32::INFINITY;
        let mut max_v = f32::NEG_INFINITY;

        let curve_dist_fn =
            CurveDistFn::new([&self.curves[0], &self.curves[1]]);

        let mut element_mesh =
            ElementMesh::from_points((&x_points, &y_points), |point| {
                let value = curve_dist_fn.eval(*point);

                // TODO: Should these just be properties on ElementMesh?
                min_v = min_v.min(value);
                max_v = max_v.max(value);

                value
            });

        let num_isolines = 10;
        let isoline_thresholds = (0..num_isolines)
            .map(|w_idx| {
                1. / ((num_isolines + 1) as Dist) * ((w_idx + 1) as Dist)
            })
            .map(|w| min_v + (max_v - min_v) * w)
            .collect_vec();

        let should_refine_triangle = |triangle: [&Vertex<Dist>; 3]| -> bool {
            isoline_thresholds.iter().any(|&threshold_value| {
                isolines::analyze_triangle(triangle, threshold_value)
                    .map(|[v0, v1]| {
                        let should_refine_vertex = |v: Vertex<Dist>| {
                            let grad =
                                curve_dist_fn.eval_gradient_magnitude(v.point);
                            let true_value = curve_dist_fn.eval(v.point);
                            let error = (v.value - true_value).abs();

                            // TODO: Does this actually make sense?
                            error > 0.05 * grad
                        };

                        should_refine_vertex(v0)
                            || should_refine_vertex(v1)
                            || should_refine_vertex(v0.mix(v1, 0.5))
                    })
                    .unwrap_or(false)
            })
        };

        element_mesh
            .refine(&|&p| curve_dist_fn.eval(p), should_refine_triangle);

        // Build isoline data
        let mut isoline_vertex_data: Vec<Vertex<Dist>> = isoline_thresholds
            .into_iter()
            .flat_map(|threshold| {
                BuildIsolines::new(
                    element_mesh.iter_triangle_vertices(),
                    threshold,
                )
            })
            .flatten()
            .collect();

        if show_mesh {
            isoline_vertex_data.extend(
                element_mesh
                    .iter_triangle_vertices()
                    .flat_map(|[v1, v2, v3]| [v1, v2, v2, v3, v3, v1])
                    .copied(),
            );
        }

        fn upload_buffer_data<T>(
            context: &WebGl2RenderingContext,
            buffer: &WebGlBuffer,
            src_data: &Vec<T>,
            target: u32,
            usage: u32,
        ) {
            context.bind_buffer(target, Some(&buffer));
            unsafe {
                // SAFETY: We're creating a view directly into memory, which might
                // become invalid if we're doing any allocations after this. The
                // view is used immediately to copy data into a GPU buffer, after
                // which it is discarded.
                let array_buffer_view =
                    js_sys::Uint8Array::view(src_data.as_u8_slice());
                context.buffer_data_with_array_buffer_view(
                    target,
                    &array_buffer_view,
                    usage,
                );
            }
        }

        let vertex_data = element_mesh.vertices();
        let index_data = &element_mesh
            .iter_triangle_elements()
            .flatten()
            .map(|idx| idx as u32)
            .collect_vec();

        // Upload updated vertex data
        upload_buffer_data(
            &self.context,
            &self.vertex_buffer,
            vertex_data,
            WebGl2RenderingContext::ARRAY_BUFFER,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        // Upload updated index data
        upload_buffer_data(
            &self.context,
            &self.index_buffer,
            index_data,
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        // Upload vertex data for isolines
        upload_buffer_data(
            &self.context,
            &self.isoline_vertex_buffer,
            &isoline_vertex_data,
            WebGl2RenderingContext::ARRAY_BUFFER,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        // Upload min/max values range
        self.context
            .uniform2f(Some(&self.value_range_uniform), min_v, max_v);

        // Upload transformation matrix
        let max_x = self.curves[0].total_length();
        let max_y = self.curves[1].total_length();

        #[rustfmt::skip]
        let transform: [f32; 16] = [
            2.0 / max_x, 0.0, 0.0, -1.0,
            0.0, 2.0 / max_y, 0.0, -1.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_uniform),
            false,
            &transform,
        );

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
            Some(&self.color_map_uniform),
            &color_map_data,
        );

        // Draw
        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        context.bind_vertex_array(Some(&self.vao_triangles));
        context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            index_data.len() as i32,
            WebGl2RenderingContext::UNSIGNED_INT,
            0,
        );

        context.bind_vertex_array(Some(&self.vao_isolines));
        context.draw_arrays(
            WebGl2RenderingContext::LINES,
            0,
            isoline_vertex_data.len() as i32,
        );

        context.bind_vertex_array(None);
    }

    pub fn update_curves(&mut self, curve_1: &JsCurve, curve_2: &JsCurve) {
        self.curves = [curve_1.clone().into(), curve_2.clone().into()];
    }

    pub fn resize(&self, width: i32, height: i32) {
        self.context.viewport(0, 0, width, height);
    }
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| "Unable to create shader object".to_string())?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error compiling shader".to_string()))
    }
}

fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| "Unable to create program object".to_string())?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error linking program".to_string()))
    }
}
