use nalgebra::{Point, RealField};

use crate::math::function::Function;

pub trait PartialDerivative<'f, T: RealField, const D: usize>:
    Function<'f, Point<T, D>, Output = T>
{
    type Output: Function<'f, Point<T, D>, Output = T>;

    fn partial_derivative(
        &'f self,
        var_idx: usize,
    ) -> <Self as PartialDerivative<'f, T, D>>::Output;
}

pub struct CentralDifferencePartialDerivativePlan<
    'f,
    T: RealField,
    const D: usize,
    F,
> where
    F: Function<'f, Point<T, D>, Output = T>,
{
    pub function: &'f F,
    pub spacing: T,
    pub var_idx: usize,
}

impl<'f, T: RealField, const D: usize, F> Function<'f, Point<T, D>>
    for CentralDifferencePartialDerivativePlan<'f, T, D, F>
where
    F: Function<'f, Point<T, D>, Output = T>,
{
    type Output = T;

    fn eval(&self, point: Point<T, D>) -> Self::Output {
        let mut prev_point = point.clone();
        prev_point[self.var_idx] -= self.spacing.clone();

        let mut next_point = point.clone();
        next_point[self.var_idx] += self.spacing.clone();

        (self.function.eval(next_point) - self.function.eval(prev_point))
            / (self.spacing.clone() + self.spacing.clone())
    }
}
