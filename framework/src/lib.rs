use lazy_static::lazy_static;

pub mod asset_id;
pub mod asset_library;
pub mod buffer;
pub mod framebuffer;
pub mod framework;
pub mod math;
pub mod mesh;
pub mod renderer;
pub mod scene;
pub mod shader;
pub mod texture;

pub use asset_id::*;
pub use asset_library::AssetsLibrary;
pub use buffer::{Buffer, BufferConfiguration, BufferType};
pub use framebuffer::*;
pub use framework::AdapterCreationError;
pub use framework::Framework;
pub use math::*;
pub use mesh::Index;
pub use mesh::Indices;
pub use mesh::Mesh;
pub use mesh::MeshConstructionDetails;
pub use mesh::MeshInstance2D;
pub use mesh::Vertex;
pub use mesh::Vertices;
pub use scene::*;
pub use texture::*;

lazy_static! {
    pub(crate) static ref FRAMEWORK_INIT_TIME: std::time::Instant = std::time::Instant::now();
}
