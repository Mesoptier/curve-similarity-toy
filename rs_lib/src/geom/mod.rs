use self::curve::Curve;
use self::point::Point;
use wasm_bindgen::prelude::*;

pub mod curve;
pub mod point;

pub type Dist = f32;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_CUSTOM_SECTION: &'static str = r#"
export type IPoint = { x:number, y: number };
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
        let points: Vec<Point> =
            serde_wasm_bindgen::from_value(points.into()).unwrap();
        Self(Curve::from_points(points))
    }

    pub fn with_point(&self, point: IPoint) -> Self {
        let point = serde_wasm_bindgen::from_value(point.into()).unwrap();

        let mut new_self = self.clone();
        new_self.0.push(point);
        new_self
    }

    pub fn at(&self, length: Dist) -> IPoint {
        serde_wasm_bindgen::to_value(&self.0.at(length))
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
