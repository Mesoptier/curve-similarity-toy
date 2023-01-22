use crate::geom::curve::Curve;
use crate::geom::Dist;
use crate::geom::point::Point;

pub struct CurveDistFn<'a> {
    curves: [&'a Curve; 2],
}

impl<'a> CurveDistFn<'a> {
    pub fn new(curves: [&'a Curve; 2]) -> Self {
        Self { curves }
    }
}

impl<'a> CurveDistFn<'a> {
    pub fn eval(&self, p: &Point) -> Dist {
        let [c1, c2] = self.curves;

        let p1 = c1.at(p.x);
        let p2 = c2.at(p.y);
        p1.dist(&p2)
    }
}
