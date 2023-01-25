use nalgebra::{Point, RealField, SVector};

use crate::math::function::Function;
use crate::math::partial_derivative::PartialDerivative;

pub trait Gradient<'f, T: RealField, const D: usize>:
    PartialDerivative<'f, T, D>
{
    type Output: Function<'f, Point<T, D>, Output = SVector<T, D>>;

    fn gradient(&'f self) -> <Self as Gradient<'f, T, D>>::Output;
}

impl<'f, T: RealField, const D: usize, F> Gradient<'f, T, D> for F
where
    F: PartialDerivative<'f, T, D>,
{
    type Output = GradientPlan<D, <F as PartialDerivative<'f, T, D>>::Output>;

    fn gradient(&'f self) -> <Self as Gradient<'f, T, D>>::Output {
        GradientPlan {
            partial_derivatives: std::array::from_fn(|var_idx| {
                self.partial_derivative(var_idx)
            }),
        }
    }
}

pub struct GradientPlan<const D: usize, F> {
    partial_derivatives: [F; D],
}

impl<'f, T: RealField, const D: usize, F> Function<'f, Point<T, D>>
    for GradientPlan<D, F>
where
    F: Function<'f, Point<T, D>, Output = T>,
{
    type Output = SVector<T, D>;

    fn eval(&'f self, p: Point<T, D>) -> Self::Output {
        SVector::from_fn(|var_idx, _| {
            self.partial_derivatives[var_idx].eval(p.clone())
        })
    }
}
