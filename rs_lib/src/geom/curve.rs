use crate::geom::point::Point;
use num::traits::NumAssignOps;
use num::{clamp, Float};
use std::fmt::Display;

#[derive(Debug, Default)]
pub struct Curve<T> {
    points: Vec<Point<T>>,
    cumulative_lengths: Vec<T>,
}

impl<T: Float + NumAssignOps + Display> Curve<T> {
    fn compute_cumulative_lengths(points: &Vec<Point<T>>) -> Vec<T> {
        (0..points.len())
            .scan(T::zero(), |cumulative_length, idx| {
                if idx != 0 {
                    *cumulative_length += points[idx].dist(&points[idx - 1]);
                }
                Some(*cumulative_length)
            })
            .collect()
    }

    pub fn from_points(points: Vec<Point<T>>) -> Self {
        Self {
            cumulative_lengths: Self::compute_cumulative_lengths(&points),
            points,
        }
    }

    pub fn total_length(&self) -> T {
        *self.cumulative_lengths.last().unwrap()
    }

    pub fn at(&self, length: T) -> Point<T> {
        let length = clamp(length, T::zero(), self.total_length());

        let idx = self
            .cumulative_lengths
            .partition_point(|&cumulative_length| cumulative_length < length);

        if idx == 0 {
            return *self.points.first().unwrap();
        } else {
            let length_1 = self.cumulative_lengths[idx - 1];
            let length_2 = self.cumulative_lengths[idx];
            assert!(length_1 <= length && length <= length_2);

            let t = (length - length_1) / (length_2 - length_1);

            let point_1 = self.points[idx - 1];
            let point_2 = self.points[idx];

            point_1 * (T::one() - t) + point_2 * t
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn curve_at() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.5, 0.0),
            Point::new(2.0, 0.0),
        ];
        let curve = Curve::from_points(points);

        assert_relative_eq!(curve.at(0.0), Point::new(0.0, 0.0));
        assert_relative_eq!(curve.at(0.5), Point::new(0.5, 0.0));
        assert_relative_eq!(curve.at(1.0), Point::new(1.0, 0.0));
        assert_relative_eq!(curve.at(1.5), Point::new(1.5, 0.0));
        assert_relative_eq!(curve.at(1.8), Point::new(1.8, 0.0));
        assert_relative_eq!(curve.at(2.0), Point::new(2.0, 0.0));
    }
}
