use std::ops::{Add, Mul};

use itertools::{Itertools, TupleWindows};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader,
    WebGlUniformLocation, WebGlVertexArrayObject,
};

use crate::geom::curve::Curve;
use crate::geom::JsCurve;
use crate::traits::mix::Mix;
use crate::traits::vec_ext::VecExt;

mod color_maps;
mod geom;
mod traits;

const BYTES_PER_FLOAT: i32 = 4;

const FLOATS_PER_POSITION: i32 = 2;
const FLOATS_PER_VALUE: i32 = 1;
const FLOATS_PER_VERTEX: i32 = FLOATS_PER_POSITION + FLOATS_PER_VALUE;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Vertex {
    x: f32,
    y: f32,
    v: f32,
}

impl Add for Vertex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vertex {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            v: self.v + rhs.v,
        }
    }
}

impl Mul<f32> for Vertex {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Vertex {
            x: self.x * rhs,
            y: self.y * rhs,
            v: self.v * rhs,
        }
    }
}

// TODO: Should this contain references to vertices, or just copies of the vertices?
#[derive(Debug)]
struct Triangle<'a>(&'a Vertex, &'a Vertex, &'a Vertex);

struct TriangleStripIter<'a, I>
where
    I: Iterator<Item = &'a u32>,
{
    vertices: &'a Vec<Vertex>,
    indices_iter: TupleWindows<I, (&'a u32, &'a u32, &'a u32)>,
}

impl<'a, I> TriangleStripIter<'a, I>
where
    I: Iterator<Item = &'a u32>,
{
    fn new(vertices: &'a Vec<Vertex>, indices: I) -> Self {
        Self {
            vertices,
            indices_iter: indices.tuple_windows(),
        }
    }
}

impl<'a, I> Iterator for TriangleStripIter<'a, I>
where
    I: Iterator<Item = &'a u32>,
{
    type Item = Triangle<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Make this actually iterate like WebGL's triangle strip
        self.indices_iter.next().map(|(i1, i2, i3)| {
            Triangle(
                &self.vertices[*i1 as usize],
                &self.vertices[*i2 as usize],
                &self.vertices[*i3 as usize],
            )
        })
    }
}

struct TriangleMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl TriangleMesh {
    fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    fn iter_triangles(
        &self,
    ) -> TriangleStripIter<'_, std::slice::Iter<'_, u32>> {
        TriangleStripIter::new(&self.vertices, self.indices.iter())
    }
}

fn make_isolines(mesh: &TriangleMesh, threshold: f32) -> Vec<Vertex> {
    let mut isoline_vertices = vec![];

    fn get_t(min: f32, max: f32, x: f32) -> f32 {
        if min > max {
            return 1.0 - get_t(max, min, x);
        }
        (x - min) / (max - min)
    }

    let mut push_edge = |v1: Vertex, v2: Vertex| {
        let t = get_t(v1.v, v2.v, threshold);
        isoline_vertices.push(v1.mix(v2, t));
    };

    for triangle in mesh.iter_triangles() {
        let Triangle(&v0, &v1, &v2) = triangle;

        match (v0.v > threshold, v1.v > threshold, v2.v > threshold) {
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

/// Builds the lattice of vertices used by the triangle mesh.
fn build_vertex_data<F: FnMut(f32, f32) -> f32>(
    x_len: u32,
    y_len: u32,
    mut f: F,
) -> Vec<Vertex> {
    let vertex_data_len = (x_len * y_len) as usize;
    let mut vertex_data: Vec<Vertex> = Vec::with_capacity(vertex_data_len);

    for y_idx in 0..y_len {
        for x_idx in 0..x_len {
            let x: f32 = x_idx as f32 / (x_len - 1) as f32;
            let y: f32 = y_idx as f32 / (y_len - 1) as f32;

            // TODO: Scale x,y to curve lengths?
            let v = f(x, y);

            let x = x * 2.0 - 1.0;
            let y = y * 2.0 - 1.0;

            vertex_data.push(Vertex { x, y, v });
        }
    }

    assert_eq!(vertex_data.len(), vertex_data_len);

    vertex_data
}

/// Builds the list of indices (of vertices in the mesh) to be interpreted as a triangle strip by
/// WebGL.
fn build_index_data(x_len: u32, y_len: u32) -> Vec<u32> {
    let index_data_len = 2 * (x_len * (y_len - 1) + (y_len - 2)) as usize;
    let mut index_data: Vec<u32> = Vec::with_capacity(index_data_len);

    for y in 0..(y_len - 1) {
        if y > 0 {
            // Degenerate begin: repeat first vertex
            index_data.push(y * x_len);
        }

        for x in 0..x_len {
            index_data.push((y * x_len) + x);
            index_data.push(((y + 1) * x_len) + x);
        }

        if y < y_len - 2 {
            // Degenerate end: repeat last vertex
            index_data.push(((y + 1) * x_len) + (x_len - 1));
        }
    }

    assert_eq!(index_data.len(), index_data_len);

    index_data
}

#[wasm_bindgen(getter_with_clone)]
pub struct Plotter {
    context: WebGl2RenderingContext,
    curves: [Curve; 2],

    color_map_uniform: WebGlUniformLocation,
    value_range_uniform: WebGlUniformLocation,

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
        context.vertex_attrib1f(value_attribute, 0.0);

        context.bind_vertex_array(None);

        Ok(Self {
            context,
            curves: [Curve::default(), Curve::default()],

            color_map_uniform,
            value_range_uniform,

            vertex_buffer,
            index_buffer,
            isoline_vertex_buffer,

            vao_triangles,
            vao_isolines,
        })
    }

    fn update_buffers(&self, x_len: u32, y_len: u32) {
        let mut mesh = TriangleMesh::new();

        let isoline_threshold = 0.5;

        let mut min_v = f32::INFINITY;
        let mut max_v = f32::NEG_INFINITY;

        // Rebuild vertex data
        mesh.vertices = build_vertex_data(x_len, y_len, |x, y| {
            let [c1, c2] = &self.curves;

            let p1 = c1.at(x * c1.total_length());
            let p2 = c2.at(y * c2.total_length());
            let v = p1.dist(&p2);

            min_v = min_v.min(v);
            max_v = max_v.max(v);

            v
        });

        // Rebuild index data
        mesh.indices = build_index_data(x_len, y_len);

        // Build isoline data
        let isoline_vertex_data = make_isolines(&mesh, isoline_threshold);

        // Upload min/max values range
        self.context
            .uniform2f(Some(&self.value_range_uniform), min_v, max_v);

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

        // Upload updated vertex data
        upload_buffer_data(
            &self.context,
            &self.vertex_buffer,
            &mesh.vertices,
            WebGl2RenderingContext::ARRAY_BUFFER,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        // Upload updated index data
        upload_buffer_data(
            &self.context,
            &self.index_buffer,
            &mesh.indices,
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
    }

    pub fn draw(&self) {
        let context = &self.context;

        let canvas: web_sys::HtmlCanvasElement = context
            .canvas()
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let res = 8;
        let x_len = canvas.width() / res;
        let y_len = canvas.height() / res;

        self.update_buffers(x_len, y_len);

        // Update color map
        context.uniform3fv_with_f32_array(
            Some(&self.color_map_uniform),
            &color_maps::COLOR_MAP_MAGMA,
        );

        let index_data_len = 2 * (x_len * (y_len - 1) + (y_len - 2)) as usize;

        // Draw
        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        context.bind_vertex_array(Some(&self.vao_triangles));
        context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            index_data_len as i32,
            WebGl2RenderingContext::UNSIGNED_INT,
            0,
        );

        // context.bind_vertex_array(Some(&self.vao_isolines));
        // context.draw_arrays(
        //     WebGl2RenderingContext::LINES,
        //     0,
        //     isoline_vertex_data.len() as i32,
        // );

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
