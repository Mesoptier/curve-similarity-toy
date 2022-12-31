use super::Dist;
use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, PartialEq)]
pub struct Point {
    pub x: Dist,
    pub y: Dist,
}

impl Point {
    pub fn new(x: Dist, y: Dist) -> Self {
        Self { x, y }
    }

    /// Computes Euclidean distance between the two points.
    pub fn dist(&self, other: &Point) -> Dist {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<Dist> for Point {
    type Output = Point;

    fn mul(self, rhs: Dist) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl AbsDiffEq for Point {
    type Epsilon = <Dist as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        Dist::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        Dist::abs_diff_eq(&self.x, &other.x, epsilon)
            && Dist::abs_diff_eq(&self.y, &other.y, epsilon)
    }
}

impl RelativeEq for Point {
    fn default_max_relative() -> Self::Epsilon {
        Dist::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        Dist::relative_eq(&self.x, &other.x, epsilon, max_relative)
            && Dist::relative_eq(&self.y, &other.y, epsilon, max_relative)
    }
}

impl UlpsEq for Point {
    fn default_max_ulps() -> u32 {
        Dist::default_max_ulps()
    }

    fn ulps_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_ulps: u32,
    ) -> bool {
        Dist::ulps_eq(&self.x, &other.x, epsilon, max_ulps)
            && Dist::ulps_eq(&self.y, &other.y, epsilon, max_ulps)
    }
}
