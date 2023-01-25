use crate::Mix;

use super::point::Point;
use super::Dist;

#[derive(Debug, Default, Clone)]
pub struct Curve {
    points: Vec<Point>,
    cumulative_lengths: Vec<Dist>,
}

impl Curve {
    fn compute_cumulative_lengths(points: &Vec<Point>) -> Vec<Dist> {
        (0..points.len())
            .scan(0., |cumulative_length, idx| {
                if idx != 0 {
                    *cumulative_length += points[idx].dist(&points[idx - 1]);
                }
                Some(*cumulative_length)
            })
            .collect()
    }

    pub fn from_points(points: Vec<Point>) -> Self {
        Self {
            cumulative_lengths: Self::compute_cumulative_lengths(&points),
            points,
        }
    }

    pub fn push(&mut self, point: Point) {
        let new_length =
            match (self.points.last(), self.cumulative_lengths.last()) {
                (Some(last_point), Some(last_length)) => {
                    *last_length + point.dist(last_point)
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

    pub fn at(&self, length: Dist) -> Point {
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

    pub fn points(&self) -> &Vec<Point> {
        &self.points
    }

    pub fn cumulative_lengths(&self) -> &Vec<Dist> {
        &self.cumulative_lengths
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn curve_at() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.5, y: 0.0 },
            Point { x: 2.0, y: 0.0 },
        ];
        let curve = Curve::from_points(points);

        assert_relative_eq!(curve.at(0.0), Point { x: 0.0, y: 0.0 });
        assert_relative_eq!(curve.at(0.5), Point { x: 0.5, y: 0.0 });
        assert_relative_eq!(curve.at(1.0), Point { x: 1.0, y: 0.0 });
        assert_relative_eq!(curve.at(1.5), Point { x: 1.5, y: 0.0 });
        assert_relative_eq!(curve.at(1.8), Point { x: 1.8, y: 0.0 });
        assert_relative_eq!(curve.at(2.0), Point { x: 2.0, y: 0.0 });
    }
}
