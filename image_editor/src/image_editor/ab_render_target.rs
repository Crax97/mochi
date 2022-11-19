use framework::{
    framework::TextureId, Framework, RgbaTexture2D, Texture, TextureConfiguration, TextureUsage,
};

enum BufferingStep {
    First,
    Second,
}

// This struct runs a render function, but every time the callback is called
// the current texture and the previous textures are swapped
pub(crate) struct ABRenderTarget {
    next_step_target: BufferingStep,
    target_1: TextureId,
    target_2: TextureId,
}

impl ABRenderTarget {
    pub fn new(width: u32, height: u32, framework: &mut Framework) -> Self {
        let target_1 = framework.allocate_texture2d(
            RgbaTexture2D::empty((width, height)),
            TextureConfiguration {
                label: Some("AB Render Target 1"),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
        let target_2 = framework.allocate_texture2d(
            RgbaTexture2D::empty((width, height)),
            TextureConfiguration {
                label: Some("AB Render Target 2"),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
        Self {
            next_step_target: BufferingStep::Second,
            target_1,
            target_2,
        }
    }

    pub fn run_render_loop<F: FnMut(&TextureId, &TextureId)>(&mut self, mut f: F) {
        let (current, previous) = self.render_targets();
        f(current, previous);
        self.advance();
    }

    pub fn result(&self) -> &TextureId {
        match self.next_step_target {
            BufferingStep::First => &self.target_1,
            BufferingStep::Second => &self.target_2,
        }
    }

    fn render_targets(&self) -> (&TextureId, &TextureId) {
        match self.next_step_target {
            BufferingStep::First => (&self.target_2, &self.target_1),
            BufferingStep::Second => (&self.target_1, &self.target_2),
        }
    }

    fn advance(&mut self) {
        match self.next_step_target {
            BufferingStep::First => {
                self.next_step_target = BufferingStep::Second;
            }
            BufferingStep::Second => {
                self.next_step_target = BufferingStep::First;
            }
        };
    }
}
