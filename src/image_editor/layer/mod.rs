mod bitmap_layer;

pub use bitmap_layer::*;
use wgpu::{CommandEncoder, RenderPass};

use crate::framework::Framework;

use super::Assets;

pub enum LayerType {
    Bitmap(bitmap_layer::BitmapLayer),
}

pub(crate) struct LayerDrawContext<'a> {
    pub render_pass: RenderPass<'a>,
    pub assets: &'a Assets,
}

impl LayerType {
    pub fn update() {}

    pub(crate) fn draw<'a>(&'a self, draw_context: &mut LayerDrawContext<'a>) {
        match &self {
            LayerType::Bitmap(bitmap_layer) => {
                let render_pass = &mut draw_context.render_pass;
                render_pass.set_pipeline(&draw_context.assets.simple_diffuse_pipeline);
                render_pass.set_bind_group(0, bitmap_layer.binding_group(), &[]);
                draw_context.assets.quad_mesh.draw(render_pass, 1);
            }
        }
    }
}
