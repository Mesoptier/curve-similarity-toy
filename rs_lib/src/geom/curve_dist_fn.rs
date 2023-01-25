use nalgebra::Point;

use crate::geom::curve::Curve;
use crate::geom::Dist;
use crate::math::function::Function;
use crate::math::partial_derivative::{
    CentralDifferencePartialDerivativePlan, PartialDerivative,
};
use crate::math::scalar_field::ScalarField;

pub struct CurveDistFn<'f> {
    curves: [&'f Curve; 2],
}

impl<'f> CurveDistFn<'f> {
    pub fn new(curves: [&'f Curve; 2]) -> Self {
        Self { curves }
    }
}

impl<'f> Function<'f, Point<Dist, 2>> for CurveDistFn<'f> {
    type Output = Dist;

    fn eval(&self, p: Point<Dist, 2>) -> Self::Output {
        let [c1, c2] = self.curves;
        let p1 = c1.eval(p.x);
        let p2 = c2.eval(p.y);
        (p1 - p2).norm()
    }
}

impl<'f> ScalarField<'f, Dist, 2> for CurveDistFn<'f> {}

impl<'f> PartialDerivative<'f, Dist, 2> for CurveDistFn<'f> {
    type Output = CentralDifferencePartialDerivativePlan<'f, Dist, 2, Self>;

    fn partial_derivative(
        &'f self,
        var_idx: usize,
    ) -> <Self as PartialDerivative<'f, Dist, 2>>::Output {
        CentralDifferencePartialDerivativePlan {
            function: self,
            spacing: 0.01,
            var_idx,
        }
    }
}
