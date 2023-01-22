use crate::geom::curve::Curve;
use crate::geom::Dist;
use crate::geom::point::Point;
use crate::pnt;

pub struct CurveDistFn<'a> {
    curves: [&'a Curve; 2],
}

impl<'a> CurveDistFn<'a> {
    pub fn new(curves: [&'a Curve; 2]) -> Self {
        Self { curves }
    }
}

impl<'a> CurveDistFn<'a> {
    pub fn eval(&self, p: Point) -> Dist {
        let [c1, c2] = self.curves;

        let p1 = c1.at(p.x);
        let p2 = c2.at(p.y);
        p1.dist(&p2)
    }

    pub fn eval_partial_derivative_y(&self, p: Point) -> Dist {
        let h = 0.01;
        (self.eval(pnt!(p.x, p.y + h)) - self.eval(pnt!(p.x, p.y - h)))
            / (2.0 * h)
    }

    pub fn eval_partial_derivative_x(&self, p: Point) -> Dist {
        let h = 0.01;
        (self.eval(pnt!(p.x + h, p.y)) - self.eval(pnt!(p.x - h, p.y)))
            / (2.0 * h)
    }

    pub fn eval_gradient_magnitude(&self, p: Point) -> Dist {
        (self.eval_partial_derivative_x(p).powi(2)
            + self.eval_partial_derivative_y(p).powi(2))
        .sqrt()
    }
}
