use framework::Mesh;
use wgpu::RenderPipeline;

pub struct AssetsLibrary {
    pub quad_mesh: Mesh,
    pub final_present_pipeline: RenderPipeline,
}
