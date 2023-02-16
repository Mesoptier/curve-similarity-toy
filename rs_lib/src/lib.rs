use std::iter;

use itertools::Itertools;
use nalgebra::{vector, Matrix4};
use ouroboros::self_referencing;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use crate::geom::curve::Curve;
use crate::geom::curve_dist_fn::CurveDistFn;
use crate::geom::{Dist, JsCurve};
use crate::math::function::Function;
use crate::math::gradient::Gradient;
use crate::plot::element_mesh::{ElementMesh, Vertex};
use crate::plot::isolines;
use crate::plot::isolines::BuildIsolines;
use crate::plot::layers::contour_lines::ContourLinesLayer;
use crate::plot::layers::density::DensityLayer;
use crate::traits::mix::Mix;

mod geom;
mod math;
mod plot;
mod traits;
mod webgl;

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format!($($t)*).into()))
}

const BYTES_PER_FLOAT: i32 = 4;

const FLOATS_PER_POSITION: i32 = 2;
const FLOATS_PER_VALUE: i32 = 1;
const FLOATS_PER_VERTEX: i32 = FLOATS_PER_POSITION + FLOATS_PER_VALUE;

fn subdivide_lengths(
    lengths: &Vec<Dist>,
    res: Dist,
    [min, max]: [Dist; 2],
) -> Vec<Dist> {
    if lengths.is_empty() {
        return vec![];
    }

    let lengths =
        iter::once(*lengths.first().unwrap())
            .chain(lengths.iter().tuple_windows::<(_, _)>().flat_map(
                |(l1, l2)| {
                    // Note: since `num_subdivisions` is 0 if both lengths are equal, this
                    // effectively also deduplicates the lengths
                    let num_subdivisions = ((l2 - l1) / res).ceil() as usize;
                    (0..num_subdivisions).map(move |i| {
                        let t = (i + 1) as Dist / (num_subdivisions as Dist);
                        l1 * (1. - t) + l2 * t
                    })
                },
            ))
            .collect_vec();

    let lo = lengths.partition_point(|&length| length <= min).max(1) - 1;
    let hi = lengths
        .partition_point(|&length| length <= max)
        .min(lengths.len() - 1);
    lengths[lo..=hi].to_vec()
}

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_CUSTOM_SECTION: &'static str = r#"
export type IDrawOptions = {
    show_mesh: boolean;
    x_bounds: [min: number, max: number];
    y_bounds: [min: number, max: number];
    draw_width: number;
    draw_height: number;
    canvas_width: number;
    canvas_height: number;
    device_pixel_ratio: number;
};
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IDrawOptions")]
    pub type IDrawOptions;
}

#[derive(Deserialize)]
struct DrawOptions {
    show_mesh: bool,
    x_bounds: [f32; 2],
    y_bounds: [f32; 2],
    draw_width: i32,
    draw_height: i32,
    canvas_width: i32,
    canvas_height: i32,
    device_pixel_ratio: f32,
}

#[self_referencing]
struct ContextWithLayers {
    context: WebGl2RenderingContext,

    #[borrows(context)]
    #[covariant]
    density_layer: DensityLayer<'this>,

    #[borrows(context)]
    #[covariant]
    contour_lines_layer: ContourLinesLayer<'this>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct Plotter {
    curves: [Curve; 2],
    context_with_layers: ContextWithLayers,
}

#[wasm_bindgen]
impl Plotter {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &WebGl2RenderingContext) -> Result<Plotter, JsValue> {
        let context = context.clone();

        // Enable blending
        context.enable(WebGl2RenderingContext::BLEND);
        context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        let context_with_layers = ContextWithLayersTryBuilder {
            context,
            density_layer_builder: |context| DensityLayer::new(context),
            contour_lines_layer_builder: |context| {
                ContourLinesLayer::new(context)
            },
        }
        .try_build()?;

        Ok(Self {
            curves: [Curve::default(), Curve::default()],
            context_with_layers,
        })
    }

    pub fn draw(&self, options: IDrawOptions) {
        self._draw(serde_wasm_bindgen::from_value(options.into()).unwrap())
    }

    fn _draw(&self, options: DrawOptions) {
        let DrawOptions {
            show_mesh,
            x_bounds,
            y_bounds,
            draw_width,
            draw_height,
            canvas_width,
            canvas_height,
            device_pixel_ratio,
            ..
        } = options;

        let context = self.context_with_layers.borrow_context();

        context.viewport(0, canvas_height - draw_height, draw_width, draw_height);

        if let Ok(range) = context
            .get_parameter(WebGl2RenderingContext::ALIASED_LINE_WIDTH_RANGE)
        {
            let range: js_sys::Float32Array = range.into();
            let min_line_width = range.at(0).unwrap();
            let max_line_width = range.at(1).unwrap();
            let line_width =
                device_pixel_ratio.clamp(min_line_width, max_line_width);
            context.line_width(line_width);
        }

        // Build mesh
        let res = 64.;
        let x_points = subdivide_lengths(
            self.curves[0].cumulative_lengths(),
            res,
            x_bounds,
        );
        let y_points = subdivide_lengths(
            self.curves[1].cumulative_lengths(),
            res,
            y_bounds,
        );

        if x_points.is_empty() || y_points.is_empty() {
            return;
        }

        let curve_dist_fn =
            CurveDistFn::new([&self.curves[0], &self.curves[1]]);
        let gradient_fn = curve_dist_fn.gradient();

        let min_value = curve_dist_fn.min_dist();
        let max_value = curve_dist_fn.max_dist();

        let mut element_mesh =
            ElementMesh::from_points((&x_points, &y_points), &curve_dist_fn);

        let num_isolines = 10;
        let isoline_thresholds = (0..num_isolines)
            .map(|w_idx| {
                1. / ((num_isolines + 1) as Dist) * ((w_idx + 1) as Dist)
            })
            .map(|w| min_value + (max_value - min_value) * w)
            .collect_vec();

        let isoline_precision = 0.2;

        let should_refine_triangle = |triangle: [&Vertex<Dist>; 3]| -> bool {
            isoline_thresholds.iter().any(|&threshold_value| {
                isolines::analyze_triangle(triangle, threshold_value)
                    .map(|[v0, v1]| {
                        let should_refine_vertex = |v: Vertex<Dist>| {
                            let gradient_magnitude =
                                gradient_fn.eval(v.point).magnitude();
                            let true_value = curve_dist_fn.eval(v.point);
                            let error = (v.value - true_value).abs();
                            error > isoline_precision * gradient_magnitude
                        };

                        should_refine_vertex(v0)
                            || should_refine_vertex(v1)
                            || should_refine_vertex(v0.mix(v1, 0.5))
                    })
                    .unwrap_or(false)
            })
        };

        element_mesh.refine(&curve_dist_fn, should_refine_triangle);

        // Build isoline data
        let mut isoline_vertex_data: Vec<Vertex<Dist>> = isoline_thresholds
            .iter()
            .flat_map(|&threshold| {
                BuildIsolines::new(
                    element_mesh.iter_triangle_vertices(),
                    threshold,
                )
            })
            .flatten()
            .collect();

        // TODO: Add separate layer for debug mesh visualization?
        if show_mesh {
            isoline_vertex_data.extend(
                element_mesh
                    .iter_triangle_vertices()
                    .flat_map(|[v1, v2, v3]| [v1, v2, v2, v3, v3, v1])
                    .copied(),
            );
        }

        let density_layer = self.context_with_layers.borrow_density_layer();
        let contour_lines_layer =
            self.context_with_layers.borrow_contour_lines_layer();

        // TODO: Make this configurable?
        let sharp_gradient = true;
        let color_gradient = colorgrad::yl_gn_bu();

        if sharp_gradient {
            density_layer
                .update_gradient_sharp(
                    &context,
                    color_gradient,
                    num_isolines + 1,
                )
                .unwrap();
        } else {
            density_layer
                .update_gradient_smooth(&context, color_gradient)
                .unwrap();
        }

        density_layer.update_value_range(&context, [min_value, max_value]);

        // Upload transformation matrix
        let m = Matrix4::new_scaling(1.0)
            .append_translation(&vector![-x_bounds[0], -y_bounds[0], 0.0])
            .append_nonuniform_scaling(&vector![
                2.0 / (x_bounds[1] - x_bounds[0]),
                2.0 / (y_bounds[1] - y_bounds[0]),
                1.0
            ])
            .append_translation(&vector![-1.0, -1.0, 0.0]);

        density_layer.update_transform(&context, m);
        contour_lines_layer.update_transform(&context, m);

        // Draw
        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        density_layer.draw(&context, &element_mesh).unwrap();
        contour_lines_layer
            .draw(&context, isoline_vertex_data)
            .unwrap();
    }

    pub fn update_curves(&mut self, curve_1: &JsCurve, curve_2: &JsCurve) {
        self.curves = [curve_1.clone().into(), curve_2.clone().into()];
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
