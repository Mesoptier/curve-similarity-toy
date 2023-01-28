use nalgebra::{Point, RealField, Scalar};

#[derive(Clone)]
pub struct LineSegment<T: Scalar, const D: usize>([Point<T, D>; 2]);

impl<T: RealField + Copy, const D: usize> LineSegment<T, D> {
    /// See https://www.geometrictools.com/Documentation/DistanceLine3Line3.pdf
    pub fn dist_squared(&self, other: &Self) -> T {
        let [p0, p1] = self.0;
        let [q0, q1] = other.0;

        let p1_sub_p0 = p1 - p0;
        let q1_sub_q0 = q1 - q0;
        let p0_sub_q0 = p0 - q0;

        let a = p1_sub_p0.dot(&p1_sub_p0);
        let b = p1_sub_p0.dot(&q1_sub_q0);
        let c = q1_sub_q0.dot(&q1_sub_q0);
        let d = p1_sub_p0.dot(&p0_sub_q0);
        let e = q1_sub_q0.dot(&p0_sub_q0);

        let det = a * c - b * b;

        let zero = T::zero();
        let one = T::one();

        let mut s;
        let mut t;

        if det > zero {
            let bte = b * e;
            let ctd = c * d;
            if bte <= ctd {
                // s <= 0
                s = zero;
                if e <= zero {
                    // t <= 0
                    // region 6
                    t = zero;
                    let nd = -d;
                    if nd >= a {
                        s = one;
                    } else if nd > zero {
                        s = nd / a;
                    }
                    // else: s is already zero
                } else if e < c {
                    // 0 < t < 1
                    // region 5
                    t = e / c;
                } else {
                    // t >= 1
                    // region 4
                    t = one;
                    let bmd = b - d;
                    if bmd >= a {
                        s = one;
                    } else if bmd > zero {
                        s = bmd / a;
                    }
                    // else:  s is already zero
                }
            } else {
                // s > 0
                s = bte - ctd;
                if s >= det {
                    // s >= 1
                    // s = 1
                    s = one;
                    let bpe = b + e;
                    if bpe <= zero {
                        // t <= 0
                        // region 8
                        t = zero;
                        let nd = -d;
                        if nd <= zero {
                            s = zero;
                        } else if nd < a {
                            s = nd / a;
                        }
                        // else: s is already one
                    } else if bpe < c {
                        // 0 < t < 1
                        // region 1
                        t = bpe / c;
                    } else {
                        // t >= 1
                        // region 2
                        t = one;
                        let bmd = b - d;
                        if bmd <= zero {
                            s = zero;
                        } else if bmd < a {
                            s = bmd / a;
                        }
                        // else:  s is already one
                    }
                } else {
                    // 0 < s < 1
                    let ate = a * e;
                    let btd = b * d;
                    if ate <= btd {
                        // t <= 0
                        // region 7
                        t = zero;
                        let nd = -d;
                        if nd <= zero {
                            s = zero;
                        } else if nd >= a {
                            s = one;
                        } else {
                            s = nd / a;
                        }
                    } else {
                        // t > 0
                        t = ate - btd;
                        if t >= det {
                            // t >= 1
                            // region 3
                            t = one;
                            let bmd = b - d;
                            if bmd <= zero {
                                s = zero;
                            } else if bmd >= a {
                                s = one;
                            } else {
                                s = bmd / a;
                            }
                        } else {
                            // 0 < t < 1
                            // region 0
                            s /= det;
                            t /= det;
                        }
                    }
                }
            }
        } else {
            // The segments are parallel. The quadratic factors to
            //   R(s,t) = a*(s-(b/a)*t)^2 + 2*d*(s - (b/a)*t) + f
            // where a*c = b^2, e = b*d/a, f = |P0-Q0|^2, and b is not
            // zero. R is constant along lines of the form s-(b/a)*t = k
            // and its occurs on the line a*s - b*t + d = 0. This line
            // must intersect both the s-axis and the t-axis because 'a'
            // and 'b' are not zero. Because of parallelism, the line is
            // also represented by -b*s + c*t - e = 0.
            //
            // The code determines an edge of the domain [0,1]^2 that
            // intersects the minimum line, or if none of the edges
            // intersect, it determines the closest corner to the minimum
            // line. The conditionals are designed to test first for
            // intersection with the t-axis (s = 0) using
            // -b*s + c*t - e = 0 and then with the s-axis (t = 0) using
            // a*s - b*t + d = 0.

            // When s = 0, solve c*t - e = 0 (t = e/c).
            if e <= zero {
                // t <= 0
                // Now solve a*s - b*t + d = 0 for t = 0 (s = -d/a).
                t = zero;
                let nd = -d;
                if nd <= zero {
                    // s <= 0
                    // region 6
                    s = zero;
                } else if nd >= a {
                    // s >= 1
                    // region 8
                    s = one;
                } else {
                    // 0 < s < 1
                    // region 7
                    s = nd / a;
                }
            } else if e >= c {
                // t >= 1
                // Now solve a*s - b*t + d = 0 for t = 1 (s = (b-d)/a).
                t = one;
                let bmd = b - d;
                if bmd <= zero {
                    // s <= 0
                    // region 4
                    s = zero;
                } else if bmd >= a {
                    // s >= 1
                    // region 2
                    s = one;
                } else {
                    // 0 < s < 1
                    // region 3
                    s = bmd / a;
                }
            } else {
                // 0 < t < 1
                // The point (0,e/c) is on the line and domain, so we have
                // one point at which R is a minimum.
                s = zero;
                t = e / c;
            }
        }

        let c0 = p0 + p1_sub_p0 * s;
        let c1 = q0 + q1_sub_q0 * t;
        (c0 - c1).magnitude_squared()
    }

    pub fn dist(&self, other: &Self) -> T {
        self.dist_squared(other).sqrt()
    }
}

impl<T: Scalar, const D: usize> From<(Point<T, D>, Point<T, D>)>
    for LineSegment<T, D>
{
    fn from((p0, p1): (Point<T, D>, Point<T, D>)) -> Self {
        Self([p0, p1])
    }
}
