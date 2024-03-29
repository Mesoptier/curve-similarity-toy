use nalgebra::Point;
use wasm_bindgen::prelude::*;

use crate::math::function::Function;

use self::curve::Curve;

pub mod curve;
pub mod curve_dist_fn;
pub mod line_segment;

pub type Dist = f32;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_CUSTOM_SECTION: &'static str = r#"
export type IPoint = [x: number, y: number];
export type IPoints = IPoint[];
export type ILengths = number[];
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IPoint")]
    pub type IPoint;
    #[wasm_bindgen(typescript_type = "IPoints")]
    pub type IPoints;
    #[wasm_bindgen(typescript_type = "ILengths")]
    pub type ILengths;
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JsCurve(Curve);

#[wasm_bindgen]
impl JsCurve {
    #[wasm_bindgen(constructor)]
    pub fn new(points: IPoints) -> Self {
        let points: Vec<Point<Dist, 2>> =
            serde_wasm_bindgen::from_value(points.into()).unwrap();
        Self(Curve::from_points(points))
    }

    pub fn with_point(&self, point: IPoint) -> Self {
        let point = serde_wasm_bindgen::from_value(point.into()).unwrap();

        let mut new_self = self.clone();
        new_self.0.push(point);
        new_self
    }

    pub fn with_replaced_point(&self, point_idx: usize, point: IPoint) -> Self {
        let point = serde_wasm_bindgen::from_value(point.into()).unwrap();

        let mut new_points = self.0.points().clone();
        new_points[point_idx] = point;
        Self(Curve::from_points(new_points))
    }

    pub fn at(&self, length: Dist) -> IPoint {
        serde_wasm_bindgen::to_value(&self.0.eval(length))
            .unwrap()
            .into()
    }

    #[wasm_bindgen(getter)]
    pub fn points(&self) -> IPoints {
        serde_wasm_bindgen::to_value(self.0.points())
            .unwrap()
            .into()
    }

    #[wasm_bindgen(getter)]
    pub fn cumulative_lengths(&self) -> ILengths {
        serde_wasm_bindgen::to_value(self.0.cumulative_lengths())
            .unwrap()
            .into()
    }
}

impl From<JsCurve> for Curve {
    fn from(js_curve: JsCurve) -> Self {
        js_curve.0
    }
}
