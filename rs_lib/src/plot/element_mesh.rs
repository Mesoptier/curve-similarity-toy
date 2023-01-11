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
        .flat_map(|bl| {
            // Indices of the corners of this quad
            let bl = bl.1 + bl.0 * y_points.len();
            let tl = bl + 1;
            let br = bl + y_points.len();
            let tr = br + 1;

            // tl---tr
            // |   ╱ |
            // | ╱   |
            // bl---br
            [[bl, tr, br], [tr, bl, tl]]
        })
        .collect_vec();

        let num_quads = (x_points.len() - 1) * (y_points.len() - 1);
        let num_triangles = num_quads * 2;
        assert_eq!(triangle_elements.len(), num_triangles);

        Self {
            vertices,
            triangle_elements,
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
