use itertools::Itertools;

use crate::{
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

pub struct ElementMesh<Value> {
    vertices: Vec<Vertex<Value>>,
    // Isosceles triangles, with corner indices in clockwise order, with base first
    triangle_elements: Vec<[usize; 3]>,
    // Maps from edge to connected edge (where edges are `(vertex_idx, edge_idx)` pairs) or None if
    // the edge is on the mesh boundary.
    connectivity: Vec<[Option<(usize, usize)>; 3]>,
}

impl<Value> ElementMesh<Value> {
    pub fn from_points<F>(
        points: (&Vec<Dist>, &Vec<Dist>),
        mut get_value: F,
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
            value: get_value(&point),
        })
        .collect_vec();

        let num_vertices = x_points.len() * y_points.len();
        assert_eq!(vertices.len(), num_vertices);

        // Build triangle elements
        let triangle_elements = Itertools::cartesian_product(
            0..(x_points.len() - 1),
            0..(y_points.len() - 1),
        )
        .flat_map(|(x_idx, y_idx)| {
            // Get indices of the vertices at the corners of the quad:
            // tl---tr
            // |     |
            // |     |
            // bl---br
            let bl = y_idx + x_idx * y_points.len();
            let tl = bl + 1;
            let br = bl + y_points.len();
            let tr = br + 1;

            [
                //      tr
                //     ╱ |
                //   ╱   |
                // bl---br
                [bl, tr, br],
                // tl---tr
                // |   ╱
                // | ╱
                // bl
                [tr, bl, tl],
            ]
        })
        .collect_vec();

        let num_quads = (x_points.len() - 1) * (y_points.len() - 1);
        let num_triangles = num_quads * 2;
        assert_eq!(triangle_elements.len(), num_triangles);

        // Build connectivity map
        let connectivity = triangle_elements
            .iter()
            .enumerate()
            .map(|(triangle_idx, &elements)| {
                // Offsets to next triangle idx
                let offset_horizontal = 2 * (y_points.len() - 1) + 1;
                let offset_vertical = 1;

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
            })
            .collect_vec();

        // Assert that each edge in the connectivity mapping corresponds to the same edge in the
        // connected triangle
        debug_assert!(connectivity.iter().enumerate().all(
            |(triangle_idx, connections)| {
                connections
                    .iter()
                    .enumerate()
                    .all(|(edge_idx, connection)| {
                        connection.map_or(
                            true,
                            |(other_triangle_idx, other_edge_idx)| {
                                [
                                    triangle_elements[triangle_idx][edge_idx],
                                    triangle_elements[triangle_idx]
                                        [(edge_idx + 1) % 3],
                                ] == [
                                    triangle_elements[other_triangle_idx]
                                        [(other_edge_idx + 1) % 3],
                                    triangle_elements[other_triangle_idx]
                                        [other_edge_idx],
                                ]
                            },
                        )
                    })
            }
        ));

        // Assert that there is the correct number of boundary edges
        debug_assert_eq!(
            connectivity
                .iter()
                .flatten()
                .filter(|c| c.is_none())
                .count(),
            2 * (x_points.len() - 1) + 2 * (y_points.len() - 1)
        );

        Self {
            vertices,
            triangle_elements,
            connectivity,
        }
    }

    pub fn iter_triangles(
        &self,
    ) -> impl Iterator<Item = [&Vertex<Value>; 3]> + '_ {
        self.triangle_elements.iter().map(|&element| {
            element.map(|vertex_idx| &self.vertices[vertex_idx])
        })
    }

    pub fn vertices(&self) -> &Vec<Vertex<Value>> {
        &self.vertices
    }

    pub fn triangle_elements(&self) -> &Vec<[usize; 3]> {
        &self.triangle_elements
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
