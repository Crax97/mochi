pub mod asset_id;
pub mod asset_library;
pub mod buffer;
pub mod depth_stencil_texture;
pub mod framebuffer;
pub mod framework;
pub mod math;
pub mod mesh;
pub mod renderer;
pub mod scene;
pub mod shader;
pub mod texture2d;

use once_cell::sync::OnceCell;

pub use asset_id::*;
pub use asset_library::AssetsLibrary;
pub use buffer::{Buffer, BufferConfiguration, BufferType};
pub use depth_stencil_texture::*;
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
pub use texture2d::Texture2d;
pub use texture2d::Texture2dConfiguration;

// static INSTANCE: OnceCell<Framework> = OnceCell::new();

pub fn instance() -> &'static Framework {
    todo!()
}

pub fn instance_mut() -> &'static mut Framework {
    todo!()
}
