use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use num::Float;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, PartialEq)]
pub struct Point<T> {
    x: T,
    y: T,
}

impl<T: Float> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Computes Euclidean distance between the two points.
    pub fn dist(&self, other: &Point<T>) -> T {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl<T> Add<Point<T>> for Point<T>
where
    T: Add<T, Output = T> + Copy,
{
    type Output = Point<T>;

    fn add(self, rhs: Point<T>) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Mul<T> for Point<T>
where
    T: Mul<T, Output = T> + Copy,
{
    type Output = Point<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: AbsDiffEq> AbsDiffEq for Point<T>
where
    T::Epsilon: Copy,
{
    type Epsilon = T::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        T::abs_diff_eq(&self.x, &other.x, epsilon)
            && T::abs_diff_eq(&self.y, &other.y, epsilon)
    }
}

impl<T: RelativeEq> RelativeEq for Point<T>
where
    T::Epsilon: Copy,
{
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        T::relative_eq(&self.x, &other.x, epsilon, max_relative)
            && T::relative_eq(&self.y, &other.y, epsilon, max_relative)
    }
}

impl<T: UlpsEq> UlpsEq for Point<T>
where
    T::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    fn ulps_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_ulps: u32,
    ) -> bool {
        T::ulps_eq(&self.x, &other.x, epsilon, max_ulps)
            && T::ulps_eq(&self.y, &other.y, epsilon, max_ulps)
    }
}
