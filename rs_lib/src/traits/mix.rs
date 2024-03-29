use std::ops::{Add, Div, Mul, Sub};

pub trait Mix<Weight, Rhs = Self> {
    type Output;
    fn mix(self, other: Rhs, t: Weight) -> Self::Output;
}

macro_rules! mix_impl {
    ($($t:ty)*) => ($(
        impl<T, D> Mix<$t, T> for T
        where
            T: Clone + Add<D, Output = T> + Sub<T, Output = D>,
            D: Mul<$t, Output = D>,
        {
            type Output = T;

            fn mix(self, other: T, t: $t) -> Self::Output {
                self.clone() + (other - self) * t
            }
        }
    )*)
}

mix_impl! { f32 f64 }

pub trait InverseMix<Weight> {
    fn inverse_mix(self, lo: Self, hi: Self) -> Weight;
}

macro_rules! inverse_mix_impl {
    ($($t:ty)*) => ($(
        impl<T> InverseMix<$t> for T
        where
            T: Copy + Sub<T, Output = T> + Div<T, Output = $t> + PartialOrd<T>,
        {
            fn inverse_mix(self, lo: Self, hi: Self) -> $t {
                if lo > hi {
                    return 1.0 - self.inverse_mix(hi, lo);
                }
                (self - lo) / (hi - lo)
            }
        }
    )*)
}

inverse_mix_impl! { f32 f64 }
