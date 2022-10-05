use wgpu::RenderPass;
pub trait RenderPassBindable {
    fn bind(&self, pass: &mut RenderPass);
}
