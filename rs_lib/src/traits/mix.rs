use std::ops::{Add, Mul};

pub trait Mix<Weight, Rhs = Self> {
    type Output;
    fn mix(self, other: Rhs, t: Weight) -> Self::Output;
}

macro_rules! mix_impl {
    ($($t:ty)*) => ($(
        impl<T> Mix<$t, T> for T
        where
            T: Add<T, Output = T> + Mul<$t, Output = T>,
        {
            type Output = T;

            fn mix(self, other: T, t: $t) -> Self::Output {
                self * (1.0 - t) + other * t
            }
        }
    )*)
}

mix_impl! { f32 f64 }
