use crate::geom::Dist;
use crate::plot::element_mesh::Vertex;
use crate::traits::mix::{InverseMix, Mix};

pub fn analyze_triangle(
    triangle: [&Vertex<Dist>; 3],
    threshold: Dist,
) -> Option<[Vertex<Dist>; 2]> {
    let [v0, v1, v2] = triangle;

    let make_endpoint =
        |v1: &Vertex<Dist>, v2: &Vertex<Dist>| -> Vertex<Dist> {
            let t = threshold.inverse_mix(v1.value, v2.value);
            v1.mix(*v2, t)
        };

    match (
        v0.value > threshold,
        v1.value > threshold,
        v2.value > threshold,
    ) {
        (true, true, true) | (false, false, false) => None,
        (true, true, false) | (false, false, true) => {
            Some([make_endpoint(v0, v2), make_endpoint(v1, v2)])
        }
        (true, false, true) | (false, true, false) => {
            Some([make_endpoint(v0, v1), make_endpoint(v2, v1)])
        }
        (true, false, false) | (false, true, true) => {
            Some([make_endpoint(v1, v0), make_endpoint(v2, v0)])
        }
    }
}

pub struct BuildIsolines<'a, It>
where
    It: Iterator<Item = [&'a Vertex<Dist>; 3]>,
{
    it: It,
    threshold: Dist,
}

impl<'a, It> BuildIsolines<'a, It>
where
    It: Iterator<Item = [&'a Vertex<Dist>; 3]>,
{
    pub fn new(it: It, threshold: Dist) -> Self {
        Self { it, threshold }
    }
}

impl<'a, It> Iterator for BuildIsolines<'a, It>
where
    It: Iterator<Item = [&'a Vertex<Dist>; 3]>,
{
    type Item = [Vertex<Dist>; 2];

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(triangle) = self.it.next() {
            if let Some(edge) = analyze_triangle(triangle, self.threshold) {
                return Some(edge);
            }
        }
        None
    }
}
