use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    sync::Arc,
};

use crate::Mesh;
pub struct AssetsLibrary {
    meshes: Arc<HashMap<String, Arc<Mesh>>>,
}

impl AssetsLibrary {
    pub fn new() -> Self {
        Self {
            meshes: Arc::new(HashMap::new()),
        }
    }
    pub fn add_mesh(&mut self, name: &str, mesh: Mesh) {
        Arc::get_mut(&mut self.meshes)
            .unwrap()
            .insert(name.to_owned(), Arc::new(mesh));
    }
}

impl<'assetlib> AssetsLibrary {
    pub fn mesh(&self, name: &str) -> &Mesh {
        let mesh = self.meshes.get(name).expect("This mesh doesn't exist");
        mesh
    }
}

pub mod mesh_names {
    pub const QUAD: &'static str = "QUAD";
}
