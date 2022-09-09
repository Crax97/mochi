use wgpu::CommandBuffer;

use crate::framework::Framework;

pub trait RenderPass {
    fn execute(&self, framework: &Framework) -> CommandBuffer;
}
