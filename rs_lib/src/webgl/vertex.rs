use bytemuck::Pod;

pub struct VertexFormat {}

pub unsafe trait Vertex: Pod {
    fn build_bindings() -> VertexFormat;
}
