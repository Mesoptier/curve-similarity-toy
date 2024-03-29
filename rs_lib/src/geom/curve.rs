use itertools::Itertools;
use nalgebra::Point;

use crate::geom::line_segment::LineSegment;
use crate::geom::Dist;
use crate::math::function::Function;
use crate::Mix;

#[derive(Debug, Default, Clone)]
pub struct Curve {
    points: Vec<Point<Dist, 2>>,
    cumulative_lengths: Vec<Dist>,
}

impl Curve {
    fn compute_cumulative_lengths(points: &Vec<Point<Dist, 2>>) -> Vec<Dist> {
        (0..points.len())
            .scan(0., |cumulative_length, idx| {
                if idx != 0 {
                    *cumulative_length +=
                        (points[idx] - &points[idx - 1]).norm();
                }
                Some(*cumulative_length)
            })
            .collect()
    }

    pub fn from_points(points: Vec<Point<Dist, 2>>) -> Self {
        Self {
            cumulative_lengths: Self::compute_cumulative_lengths(&points),
            points,
        }
    }

    pub fn push(&mut self, point: Point<Dist, 2>) {
        let new_length =
            match (self.points.last(), self.cumulative_lengths.last()) {
                (Some(last_point), Some(last_length)) => {
                    *last_length + (point - last_point).norm()
                }
                (None, None) => 0.,
                _ => unreachable!(),
            };

        self.points.push(point);
        self.cumulative_lengths.push(new_length);
    }

    pub fn total_length(&self) -> Dist {
        *self.cumulative_lengths.last().unwrap()
    }

    pub fn points(&self) -> &Vec<Point<Dist, 2>> {
        &self.points
    }

    pub fn cumulative_lengths(&self) -> &Vec<Dist> {
        &self.cumulative_lengths
    }

    pub fn line_segments(
        &self,
    ) -> impl Iterator<Item = LineSegment<Dist, 2>> + Clone + '_ {
        self.points
            .iter()
            .cloned()
            .tuple_windows::<(_, _)>()
            .map_into()
    }
}

impl<'f> Function<'f, Dist> for Curve {
    type Output = Point<Dist, 2>;

    fn eval(&'f self, length: Dist) -> Self::Output {
        let length = length.clamp(0., self.total_length());

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

            point_1.mix(point_2, t)
        }
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use nalgebra::point;

    use super::*;

    #[test]
    fn curve_at() {
        let points = vec![
            point![0.0, 0.0],
            point![1.0, 0.0],
            point![1.5, 0.0],
            point![2.0, 0.0],
        ];
        let curve = Curve::from_points(points);

        assert_relative_eq!(curve.eval(0.0), point![0.0, 0.0]);
        assert_relative_eq!(curve.eval(0.5), point![0.5, 0.0]);
        assert_relative_eq!(curve.eval(1.0), point![1.0, 0.0]);
        assert_relative_eq!(curve.eval(1.5), point![1.5, 0.0]);
        assert_relative_eq!(curve.eval(1.8), point![1.8, 0.0]);
        assert_relative_eq!(curve.eval(2.0), point![2.0, 0.0]);
    }
}
