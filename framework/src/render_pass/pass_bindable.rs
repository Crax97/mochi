use wgpu::BindGroup;

pub trait PassBindble {
    fn bind<'s, 'call, 'pass>(&'s self, index: u32, pass: &'call mut wgpu::RenderPass<'pass>)
    where
        'pass: 'call,
        's: 'pass;
}

impl PassBindble for BindGroup {
    fn bind<'s, 'call, 'pass>(&'s self, index: u32, pass: &'call mut wgpu::RenderPass<'pass>)
    where
        'pass: 'call,
        's: 'pass,
    {
        pass.set_bind_group(index, &self, &[])
    }
}
