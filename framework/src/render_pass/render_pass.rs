use wgpu::CommandBuffer;

use super::PassBindble;

pub trait RenderPass {
    fn bind_all<'s, 'call, 'pass>(
        &'s self,
        pass: &'call mut wgpu::RenderPass<'pass>,
        items: &'call [(u32, &'pass dyn PassBindble)],
    ) where
        'pass: 'call,
        's: 'pass,
    {
        for (i, element) in items.into_iter() {
            element.bind(*i, pass);
        }
    }

    fn execute_with_renderpass<'s, 'call, 'pass>(
        &'s self,
        pass: &'call mut wgpu::RenderPass<'pass>,
        items: &'call [(u32, &'pass dyn PassBindble)],
    ) where
        'pass: 'call,
        's: 'pass;
}
