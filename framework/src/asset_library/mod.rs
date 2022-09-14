use std::{cell::RefCell, collections::HashMap, iter::FromIterator, rc::Rc};

use crate::{
    render_pass::{self, RenderPass, SimpleColoredPass, SimpleTexturedPass},
    Framework, Mesh, MeshConstructionDetails, Vertex,
};
use cgmath::{point2, point3};
pub struct AssetsLibrary {
    pipelines: HashMap<String, Box<dyn render_pass::RenderPass>>,
    meshes: HashMap<String, Mesh>,
}

impl AssetsLibrary {
    pub fn new(framework: &'_ Framework) -> Rc<RefCell<Self>> {
        let quad_mesh_vertices = [
            Vertex {
                position: point3(-1.0, 1.0, 0.0),
                tex_coords: point2(0.0, 1.0),
            },
            Vertex {
                position: point3(1.0, 1.0, 0.0),
                tex_coords: point2(1.0, 1.0),
            },
            Vertex {
                position: point3(-1.0, -1.0, 0.0),
                tex_coords: point2(0.0, 0.0),
            },
            Vertex {
                position: point3(1.0, -1.0, 0.0),
                tex_coords: point2(1.0, 0.0),
            },
        ]
        .into();

        let indices = [0u16, 1, 2, 2, 1, 3].into();
        let quad_mesh = Mesh::new(
            &framework,
            MeshConstructionDetails {
                vertices: quad_mesh_vertices,
                indices,
                allow_editing: false,
            },
        );

        let library = Rc::new(RefCell::new(Self {
            pipelines: HashMap::from_iter([].into_iter()),
            meshes: HashMap::from_iter(std::iter::once((mesh_names::QUAD.to_owned(), quad_mesh))),
        }));

        let simple_diffuse_pipeline = SimpleTexturedPass::new(framework, library.clone());
        let simple_colored_pipeline = SimpleColoredPass::new(framework, library.clone());

        library.borrow_mut().add_pipeline(
            pipeline_names::SIMPLE_TEXTURED,
            Box::new(simple_diffuse_pipeline),
        );
        library.borrow_mut().add_pipeline(
            pipeline_names::SIMPLE_COLORED,
            Box::new(simple_colored_pipeline),
        );

        library
    }
    pub fn add_pipeline(&mut self, name: &str, pipeline: Box<dyn RenderPass>) {
        self.pipelines.insert(name.to_owned(), pipeline);
    }
    pub fn add_mesh(&mut self, name: &str, mesh: Mesh) {
        self.meshes.insert(name.to_owned(), mesh);
    }
}

impl<'assetlib> AssetsLibrary {
    pub fn pipeline(&'assetlib self, name: &str) -> &'assetlib Box<dyn RenderPass> {
        self.pipelines
            .get(name)
            .expect("This pipeline doesn't exist")
    }
    pub fn mesh(&self, name: &str) -> &Mesh {
        self.meshes.get(name).expect("This mesh doesn't exist")
    }
}

pub mod pipeline_names {
    pub const SIMPLE_TEXTURED: &'static str = "SIMPLE_TEXTURED";
    pub const SIMPLE_COLORED: &'static str = "SIMPLE_COLORED";
}

pub mod mesh_names {
    pub const QUAD: &'static str = "QUAD";
}
