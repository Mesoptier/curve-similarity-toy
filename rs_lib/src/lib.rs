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
use crate::geom::point::Point;
use crate::geom::{Dist, JsCurve};
use crate::plot::element_mesh::{ElementMesh, Vertex};
use crate::traits::mix::Mix;
use crate::traits::vec_ext::VecExt;

mod geom;
mod plot;
mod traits;

macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format!($($t)*).into()))
}

const BYTES_PER_FLOAT: i32 = 4;

const FLOATS_PER_POSITION: i32 = 2;
const FLOATS_PER_VALUE: i32 = 1;
const FLOATS_PER_VERTEX: i32 = FLOATS_PER_POSITION + FLOATS_PER_VALUE;

fn make_isolines(
    mesh: &ElementMesh<Dist>,
    threshold: Dist,
) -> Vec<Vertex<Dist>> {
    let mut isoline_vertices = vec![];

    fn get_t(min: Dist, max: Dist, x: Dist) -> Dist {
        if min > max {
            return 1.0 - get_t(max, min, x);
        }
        (x - min) / (max - min)
    }

    let mut push_edge = |v1: &Vertex<Dist>, v2: &Vertex<Dist>| {
        let t = get_t(v1.value, v2.value, threshold);
        isoline_vertices.push(v1.mix(*v2, t));
    };

    for triangle in mesh.iter_triangle_vertices() {
        let [v0, v1, v2] = triangle;

        match (
            v0.value > threshold,
            v1.value > threshold,
            v2.value > threshold,
        ) {
            (true, true, true) | (false, false, false) => {}
            (true, true, false) | (false, false, true) => {
                push_edge(v0, v2);
                push_edge(v1, v2);
            }
            (true, false, true) | (false, true, false) => {
                push_edge(v0, v1);
                push_edge(v2, v1);
            }
            (true, false, false) | (false, true, true) => {
                push_edge(v1, v0);
                push_edge(v2, v0);
            }
        }
    }

    isoline_vertices
}

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

    pub fn draw(&self) {
        let context = &self.context;

        // Build mesh
        let res = 16.;
        let x_points =
            subdivide_lengths(self.curves[0].cumulative_lengths(), res);
        let y_points =
            subdivide_lengths(self.curves[1].cumulative_lengths(), res);

        let mut min_v = f32::INFINITY;
        let mut max_v = f32::NEG_INFINITY;

        let value_at_point = |&point: &Point| -> Dist {
            let [c1, c2] = &self.curves;

            let p1 = c1.at(point.x);
            let p2 = c2.at(point.y);
            p1.dist(&p2)
        };

        let element_mesh =
            ElementMesh::from_points((&x_points, &y_points), |point| {
                let value = value_at_point(point);

                // TODO: Should these just be properties on ElementMesh?
                min_v = min_v.min(value);
                max_v = max_v.max(value);

                value
            });

        let should_refine_edge = |edge: [&Vertex<Dist>; 2]| -> bool {
            let [left, right] = edge;
            let mid_lerp = left.value.mix(right.value, 0.5);
            let mid_eval = value_at_point(&left.point.mix(right.point, 0.5));
            let error = (mid_lerp - mid_eval).abs();
            error > 0.1
        };

        // Build isoline data
        let isoline_vertex_data = [
            min_v + (max_v - min_v) * 0.25,
            min_v + (max_v - min_v) * 0.50,
            min_v + (max_v - min_v) * 0.75,
        ]
        .into_iter()
        .flat_map(|threshold| make_isolines(&element_mesh, threshold))
        .collect();

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

/*
#[wasm_bindgen]
pub fn start(
    context: &WebGl2RenderingContext,
    width: u32,
    height: u32,
) -> Result<(), JsValue> {
    start_for_real(context, width, height)
}

pub fn start_for_real(
    context: &WebGl2RenderingContext,
    _width: u32,
    _height: u32,
) -> Result<(), JsValue> {
    let context = context.clone();

    assert_eq!(
        (BYTES_PER_FLOAT * FLOATS_PER_VERTEX) as usize,
        mem::size_of::<Vertex>()
    );

    // UI
    let show_mesh = Rc::new(Cell::new(false));
    let is_playing = Rc::new(Cell::new(true));

    let color_map_name = Rc::new(RefCell::new("magma".to_string()));
    let color_map_name_changed = Rc::new(Cell::new(true));

    let resolution = Rc::new(Cell::new(7i32));
    let resolution_changed = Rc::new(Cell::new(true));

    let threshold = Rc::new(Cell::new(0.5));

    // Event listeners
    let document = web_sys::window().unwrap().document().unwrap();

    {
        let show_mesh = show_mesh.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            show_mesh.set(!show_mesh.get());
        });
        let button = document
            .get_element_by_id("toggle-show-mesh")
            .expect("Element with ID #toggle-show-mesh");
        button.add_event_listener_with_callback(
            "click",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    {
        let is_playing = is_playing.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            is_playing.set(!is_playing.get());
        });
        let button = document
            .get_element_by_id("toggle-playing")
            .expect("Element with ID #toggle-playing");
        button.add_event_listener_with_callback(
            "click",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    {
        let color_map_name = color_map_name.clone();
        let color_map_name_changed = color_map_name_changed.clone();
        let closure =
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::InputEvent| {
                *color_map_name.borrow_mut() = event
                    .current_target()
                    .unwrap()
                    .dyn_into::<web_sys::HtmlSelectElement>()
                    .unwrap()
                    .value();
                color_map_name_changed.set(true);
            });
        let select = document
            .get_element_by_id("select-color-map")
            .expect("Element with ID #select-color-map");
        select.add_event_listener_with_callback(
            "change",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    {
        let resolution = resolution.clone();
        let resolution_changed = resolution_changed.clone();
        let closure =
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::InputEvent| {
                resolution.set(
                    event
                        .current_target()
                        .unwrap()
                        .dyn_into::<web_sys::HtmlInputElement>()
                        .unwrap()
                        .value()
                        .parse()
                        .unwrap(),
                );
                resolution_changed.set(true);
            });
        let input = document
            .get_element_by_id("input-resolution")
            .expect("Element with ID #input-resolution");
        input.add_event_listener_with_callback(
            "input",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    {
        let threshold = threshold.clone();
        let closure =
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::InputEvent| {
                threshold.set(
                    event
                        .current_target()
                        .unwrap()
                        .dyn_into::<web_sys::HtmlInputElement>()
                        .unwrap()
                        .value()
                        .parse()
                        .unwrap(),
                );
            });
        let input = document
            .get_element_by_id("input-threshold")
            .expect("Element with ID #input-threshold");
        input.add_event_listener_with_callback(
            "input",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    // Begin draw loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mesh = Rc::new(RefCell::new(TriangleMesh::new()));

    // TODO: Remove this debugging code
    //    mesh.borrow_mut().vertices = build_vertex_data(4, 4, 0.0);
    //    mesh.borrow_mut().indices = build_index_data(4, 4);
    //    let msg =
    //        format!("{:#?}", mesh.borrow().iter_triangles().collect::<Vec<_>>());
    //    web_sys::console::log_1(&msg.into());
    //    let msg = format!("{:#?}", make_isolines(&mesh.borrow(), 0.5));
    //    web_sys::console::log_1(&msg.into());

    let mut frame = 0;
    *g.borrow_mut() = Some(Closure::new(move || {
        if is_playing.get() {
            frame += 1;
        }

        let t = (frame as f32) / 60.0;

        let resolution = 2i32.pow(resolution.get() as u32);
        let x_len = resolution;
        let y_len = resolution;

        let index_data_len = 2 * (x_len * (y_len - 1) + (y_len - 2)) as usize;

        if resolution_changed.get() {
            resolution_changed.set(false);

            // Rebuild vertex data
            mesh.borrow_mut().vertices = build_vertex_data(x_len, y_len, t);

            // Rebuild index data
            mesh.borrow_mut().indices = build_index_data(x_len, y_len);

            // Upload updated vertex data
            context.bind_buffer(
                WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&vertex_buffer),
            );
            unsafe {
                // SAFETY: We're creating a view directly into memory, which might
                // become invalid if we're doing any allocations after this. The
                // view is used immediately to copy data into a GPU buffer, after
                // which it is discarded.
                let vertex_data_array_buf_view = js_sys::Uint8Array::view(
                    mesh.borrow().vertices.as_u8_slice(),
                );
                context.buffer_data_with_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    &vertex_data_array_buf_view,
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                );
            }

            // Upload updated index data
            context.bind_buffer(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                Some(&index_buffer),
            );
            unsafe {
                // SAFETY: We're creating a view directly into memory, which might
                // become invalid if we're doing any allocations after this. The
                // view is used immediately to copy data into a GPU buffer, after
                // which it is discarded.
                let index_data_array_buf_view = js_sys::Uint8Array::view(
                    mesh.borrow().indices.as_u8_slice(),
                );
                context.buffer_data_with_array_buffer_view(
                    WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                    &index_data_array_buf_view,
                    WebGl2RenderingContext::STATIC_DRAW,
                )
            }
        } else {
            // Update vertex data
            update_vertex_data(t, &mut mesh.borrow_mut().vertices);

            // Upload updated vertex data
            context.bind_buffer(
                WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&vertex_buffer),
            );
            unsafe {
                // SAFETY: We're creating a view directly into memory, which might
                // become invalid if we're doing any allocations after this. The
                // view is used immediately to copy data into a GPU buffer, after
                // which it is discarded.
                let vertex_data_array_buf_view = js_sys::Uint8Array::view(
                    mesh.borrow().vertices.as_u8_slice(),
                );
                context.buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &vertex_data_array_buf_view,
                );
            }
        }

        // Build and upload vertex data for isolines
        let isoline_vertex_data =
            make_isolines(&mesh.borrow(), threshold.get());
        context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&isoline_vertex_buffer),
        );
        unsafe {
            // SAFETY: We're creating a view directly into memory, which might
            // become invalid if we're doing any allocations after this. The
            // view is used immediately to copy data into a GPU buffer, after
            // which it is discarded.
            let isoline_vertex_data_array_buf_view =
                js_sys::Uint8Array::view(isoline_vertex_data.as_u8_slice());
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &isoline_vertex_data_array_buf_view,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        }

        // Update color map if the selected one changed
        if color_map_name_changed.get() {
            color_map_name_changed.set(false);

            let color_map = match color_map_name.borrow().as_str() {
                "magma" => color_maps::COLOR_MAP_MAGMA,
                "inferno" => color_maps::COLOR_MAP_INFERNO,
                "plasma" => color_maps::COLOR_MAP_PLASMA,
                "viridis" => color_maps::COLOR_MAP_VIRIDIS,
                _ => unreachable!("Invalid color map name"),
            };

            context.uniform3fv_with_f32_array(
                Some(&color_map_uniform),
                &color_map,
            );
        }

        let finished = false;
        if finished {
            // Drop our handle to this closure so that it will get cleaned up once we return.
            let _ = f.borrow_mut().take();
            return;
        }

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}*/

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
