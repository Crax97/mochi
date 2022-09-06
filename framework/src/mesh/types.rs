use cgmath::{Point2, Point3};

pub type Index = u16;
pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub tex_coords: Point2<f32>,
}
pub struct Indices(pub Vec<Index>);
pub struct Vertices(pub Vec<Vertex>);

impl<T: as_slice::AsSlice + IntoIterator> From<T> for Indices
where
    T::Item: Into<Index>,
{
    fn from(slice: T) -> Self {
        let index_vec: Vec<Index> = slice.into_iter().map(|i| i.into()).collect();
        Self(index_vec)
    }
}

impl<T: as_slice::AsSlice + IntoIterator> From<T> for Vertices
where
    T::Item: Into<Vertex>,
{
    fn from(slice: T) -> Self {
        let vertices_vec: Vec<Vertex> = slice.into_iter().map(|i| i.into()).collect();
        Self(vertices_vec)
    }
}
