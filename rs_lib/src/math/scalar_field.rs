use nalgebra::{Point, RealField};

use crate::math::function::Function;
use crate::math::gradient::Gradient;
use crate::math::partial_derivative::PartialDerivative;

pub trait ScalarField<'f, T: RealField, const D: usize>:
    Function<'f, Point<T, D>, Output = T>
    + PartialDerivative<'f, T, D>
    + Gradient<'f, T, D>
{
}
