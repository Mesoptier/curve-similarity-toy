use itertools::Itertools;
use std::collections::VecDeque;

use crate::{
    console_log,
    geom::{point::Point, Dist},
    traits::mix::Mix,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex<Value> {
    pub point: Point,
    pub value: Value,
}

impl<Weight, Value> Mix<Weight> for Vertex<Value>
where
    Weight: Copy,
    Value: Mix<Weight, Output = Value>,
    Point: Mix<Weight, Output = Point>,
{
    type Output = Vertex<Value>;

    fn mix(self, other: Self, t: Weight) -> Self::Output {
        Self {
            point: self.point.mix(other.point, t),
            value: self.value.mix(other.value, t),
        }
    }
}

/// Triangle, with corner elements in clockwise order, with base first.
#[derive(Copy, Clone, Debug)]
struct Triangle {
    elements: [usize; 3],
    /// Map from edge index to connected edge (as `Some((triangle_idx, edge_idx))`) or `None` if the
    /// edge is on the mesh boundary.
    connectivity: [Option<(usize, usize)>; 3],
    /// Triangle degree (i.e. how many times it has been refined)
    degree: usize,
}

impl Triangle {
    fn edge(&self, edge_idx: usize) -> [usize; 2] {
        [self.elements[edge_idx], self.elements[(edge_idx + 1) % 3]]
    }
    fn edge_reverse(&self, edge_idx: usize) -> [usize; 2] {
        let mut edge = self.edge(edge_idx);
        edge.reverse();
        edge
    }

    fn edge_degree(&self, edge_idx: usize) -> usize {
        if edge_idx == 0 {
            self.degree
        } else {
            self.degree + 1
        }
    }
}

pub struct ElementMesh<Value> {
    vertices: Vec<Vertex<Value>>,
    triangles: Vec<Triangle>,
}

impl<Value> ElementMesh<Value> {
    pub fn from_points<F>(
        points: (&Vec<Dist>, &Vec<Dist>),
        mut value_at_point: F,
    ) -> Self
    where
        F: FnMut(&Point) -> Value,
    {
        let (x_points, y_points) = points;

        // Build vertices
        let vertices = Itertools::cartesian_product(
            x_points.iter().copied(),
            y_points.iter().copied(),
        )
        .map_into()
        .map(|point| Vertex {
            point,
            value: value_at_point(&point),
        })
        .collect_vec();

        let num_vertices = x_points.len() * y_points.len();
        assert_eq!(vertices.len(), num_vertices);

        // Offsets to next triangle idx
        let offset_horizontal = 2 * (y_points.len() - 1) + 1;
        let offset_vertical = 1;

        let compute_connectivity =
            |triangle_idx: usize, elements: [usize; 3]| {
                if triangle_idx % 2 == 0 {
                    let [_, _, br] = elements;
                    let br_x_idx = br / y_points.len();
                    let br_y_idx = br % y_points.len();

                    [
                        Some((triangle_idx + 1, 0)),
                        if br_x_idx < x_points.len() - 1 {
                            Some((triangle_idx + offset_horizontal, 1))
                        } else {
                            None
                        },
                        if br_y_idx > 0 {
                            Some((triangle_idx - offset_vertical, 2))
                        } else {
                            None
                        },
                    ]
                } else {
                    let [_, _, tl] = elements;
                    let tl_x_idx = tl / y_points.len();
                    let tl_y_idx = tl % y_points.len();

                    [
                        Some((triangle_idx - 1, 0)),
                        if tl_x_idx > 0 {
                            Some((triangle_idx - offset_horizontal, 1))
                        } else {
                            None
                        },
                        if tl_y_idx < y_points.len() - 1 {
                            Some((triangle_idx + offset_vertical, 2))
                        } else {
                            None
                        },
                    ]
                }
            };

        // Build triangles
        let triangles = Itertools::cartesian_product(
            0..(x_points.len() - 1),
            0..(y_points.len() - 1),
        )
        .enumerate()
        .flat_map(|(quad_idx, (x_idx, y_idx))| {
            // Get indices of the vertices at the corners of the quad:
            // tl---tr
            // |     |
            // |     |
            // bl---br
            let bl = y_idx + x_idx * y_points.len();
            let tl = bl + 1;
            let br = bl + y_points.len();
            let tr = br + 1;

            let make_triangle =
                |elements: [usize; 3], triangle_idx: usize| Triangle {
                    elements,
                    connectivity: compute_connectivity(triangle_idx, elements),
                    degree: 0,
                };

            [
                //      tr
                //     ╱ |
                //   ╱   |
                // bl---br
                make_triangle([bl, tr, br], quad_idx * 2),
                // tl---tr
                // |   ╱
                // | ╱
                // bl
                make_triangle([tr, bl, tl], quad_idx * 2 + 1),
            ]
        })
        .collect_vec();

        let num_quads = (x_points.len() - 1) * (y_points.len() - 1);
        let num_triangles = num_quads * 2;
        assert_eq!(triangles.len(), num_triangles);

        // Assert that each edge in the connectivity mapping corresponds to the same edge in the
        // connected triangle
        debug_assert!(triangles.iter().all(|triangle| {
            triangle.connectivity.iter().enumerate().all(
                |(edge_idx, connection)| {
                    connection.map_or(
                        true,
                        |(other_triangle_idx, other_edge_idx)| {
                            let other_triangle = &triangles[other_triangle_idx];
                            triangle.edge(edge_idx)
                                == other_triangle.edge_reverse(other_edge_idx)
                        },
                    )
                },
            )
        }));

        // Assert that there is the correct number of boundary edges
        debug_assert_eq!(
            triangles
                .iter()
                .flat_map(|t| t.connectivity)
                .filter(|c| c.is_none())
                .count(),
            2 * (x_points.len() - 1) + 2 * (y_points.len() - 1)
        );

        Self {
            vertices,
            triangles,
        }
    }

    // TODO: Refactor this whole mess
    pub fn refine(
        &mut self,
        value_at_point: &impl Fn(&Point) -> Value,
        should_refine_triangle: impl Fn([&Vertex<Value>; 3]) -> bool,
    ) {
        #[derive(Debug)]
        struct Entry {
            triangle_idx: usize,
            triangle_degree: usize,
        }

        let mut queue = VecDeque::from_iter((0..self.triangles.len()).map(
            |triangle_idx| Entry {
                triangle_idx,
                triangle_degree: 0,
            },
        ));

        let min_degree = 1;
        let max_degree = 10;

        while let Some(entry) = queue.pop_front() {
            if entry.triangle_degree > max_degree {
                continue;
            }
            if self.triangles[entry.triangle_idx].degree
                != entry.triangle_degree
            {
                continue;
            }
            if entry.triangle_degree >= min_degree
                && !should_refine_triangle(
                    self.triangles[entry.triangle_idx]
                        .elements
                        .map(|vertex_idx| &self.vertices[vertex_idx]),
                )
            {
                continue;
            }

            let new_triangles =
                self.refine_triangle_base(entry.triangle_idx, value_at_point);
            queue.extend(new_triangles.iter().map(|&triangle_idx| Entry {
                triangle_idx,
                triangle_degree: entry.triangle_degree + 1,
            }));
        }
    }

    fn refine_triangle_base(
        &mut self,
        triangle_idx: usize,
        value_at_point: &impl Fn(&Point) -> Value,
    ) -> [usize; 2] {
        let triangle = self.triangles[triangle_idx];

        let edge = triangle
            .edge(0)
            .map(|vertex_idx| &self.vertices[vertex_idx]);

        let mid_point = edge[0].point.mix(edge[1].point, 0.5);
        let mid_value = value_at_point(&mid_point);

        let other_triangle_idx = triangle.connectivity[0].map(
            |(other_triangle_idx, other_edge_idx)| {
                if other_edge_idx == 0 {
                    other_triangle_idx
                } else {
                    // Subdivide the connected triangle and return the index of the new triangle
                    // that's now connected at the base
                    self.refine_triangle_base(
                        other_triangle_idx,
                        value_at_point,
                    )[other_edge_idx - 1]
                }
            },
        );

        assert!(other_triangle_idx
            .map(|idx| self.triangles[idx].degree == triangle.degree)
            .unwrap_or(true));

        let new_vertex_idx = self.vertices.len();
        self.vertices.push(Vertex {
            point: mid_point,
            value: mid_value,
        });

        let mut foo = |triangle_idx: usize| {
            //       2                    2
            //      ╱ ╲                  ╱|╲
            //    ╱     ╲      ->      ╱  |  ╲
            //  ╱    T    ╲          ╱  T | T' ╲
            // 1 --------- 0        1 --- m --- 0

            let triangle = self.triangles[triangle_idx];
            let new_triangle_idx = self.triangles.len();

            self.triangles[triangle_idx] = Triangle {
                elements: [
                    triangle.elements[1],
                    triangle.elements[2],
                    new_vertex_idx,
                ],
                connectivity: [
                    triangle.connectivity[1],
                    Some((new_triangle_idx, 2)),
                    None, // Connected later
                ],
                degree: triangle.degree + 1,
            };

            self.triangles.push(Triangle {
                elements: [
                    triangle.elements[2],
                    triangle.elements[0],
                    new_vertex_idx,
                ],
                connectivity: [
                    triangle.connectivity[2],
                    None, // Connected later
                    Some((triangle_idx, 1)),
                ],
                degree: triangle.degree + 1,
            });

            // Fix connectivity
            for triangle_idx in [triangle_idx, new_triangle_idx] {
                if let Some((t, e)) =
                    self.triangles[triangle_idx].connectivity[0]
                {
                    self.triangles[t].connectivity[e] = Some((triangle_idx, 0));
                }
            }

            [(triangle_idx, 2), (new_triangle_idx, 1)]
        };

        let [edge_1, edge_2] = foo(triangle_idx);

        if let Some([other_edge_1, other_edge_2]) = other_triangle_idx.map(foo)
        {
            let mut connect_edges = |e1: (usize, usize), e2: (usize, usize)| {
                assert_eq!(
                    self.triangles[e1.0].edge(e1.1),
                    self.triangles[e2.0].edge_reverse(e2.1)
                );

                self.triangles[e1.0].connectivity[e1.1] = Some(e2);
                self.triangles[e2.0].connectivity[e2.1] = Some(e1);
            };

            connect_edges(edge_1, other_edge_2);
            connect_edges(edge_2, other_edge_1);
        }

        [edge_1.0, edge_2.0]
    }

    pub fn iter_triangle_elements(
        &self,
    ) -> impl Iterator<Item = [usize; 3]> + '_ {
        self.triangles.iter().map(|triangle| triangle.elements)
    }

    pub fn iter_triangle_vertices(
        &self,
    ) -> impl Iterator<Item = [&Vertex<Value>; 3]> + '_ {
        self.iter_triangle_elements().map(|elements| {
            elements.map(|vertex_idx| &self.vertices[vertex_idx])
        })
    }

    pub fn vertices(&self) -> &Vec<Vertex<Value>> {
        &self.vertices
    }

    pub fn triangles(&self) -> &Vec<Triangle> {
        &self.triangles
    }
}

#[cfg(test)]
mod test {
    use crate::plot::element_mesh::ElementMesh;

    #[test]
    fn from_points() {
        let x_points = vec![0., 1., 2.];
        let y_points = vec![0., 1., 2.];
        ElementMesh::from_points((&x_points, &y_points), |_| 0.);
    }
}
