use cgmath::{num_traits::ToPrimitive, Vector2};
use framework::{
    Framework, Texture2d, Texture2dConfiguration, TypedBuffer, TypedBufferConfiguration,
};
use scene::{Camera2d, Camera2dUniformBlock};
use wgpu::{
    BindGroup, ImageDataLayout, Sampler, SamplerDescriptor, Texture, TextureDescriptor,
    TextureView, TextureViewDescriptor, TextureViewDimension,
};

pub struct BitmapLayerConfiguration {
    pub label: String,
    pub width: u32,
    pub initial_background_color: [f32; 4],
    pub height: u32,
}
pub struct BitmapLayer<'framework> {
    texture: Texture2d,
    configuration: BitmapLayerConfiguration,
    camera_buffer: TypedBuffer<'framework>,
    camaera_bind_group: BindGroup,
}

impl<'framework> BitmapLayer<'framework> {
    pub fn new(framework: &'framework Framework, configuration: BitmapLayerConfiguration) -> Self {
        let bytes: Vec<u32> = (1..(configuration.width * configuration.height) + 1)
            .map(|_| {
                let bg = configuration.initial_background_color;
                let r = (bg[0].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                let g = (bg[1].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                let b = (bg[2].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                let a = (bg[3].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                u32::from_le_bytes([r, g, b, a])
            })
            .collect();
        let bytes = bytemuck::cast_slice(&bytes);
        Self::new_from_bytes(framework, &bytes, configuration)
    }

    pub fn new_from_bytes(
        framework: &'framework Framework,
        bytes: &[u8],
        configuration: BitmapLayerConfiguration,
    ) -> Self {
        let dimension = wgpu::TextureDimension::D2;
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let size = wgpu::Extent3d {
            width: configuration.width,
            height: configuration.height,
            depth_or_array_layers: 1,
        };

        let mut texture = Texture2d::new(
            framework,
            Texture2dConfiguration {
                width: configuration.width,
                height: configuration.height,
                format,
                allow_cpu_write: true,
                allow_cpu_read: true,
                allow_use_as_render_target: true,
            },
        );
        texture.write_data(bytes, framework);

        let camera = TypedBuffer::new(
            framework,
            TypedBufferConfiguration::<Camera2dUniformBlock> {
                initial_setup: framework::typed_buffer::BufferInitialSetup::Data(&vec![
                    Camera2dUniformBlock::from(&Camera2d::new(
                        -0.1,
                        1000.0,
                        [
                            -(configuration.width as f32) * 0.5,
                            configuration.width as f32 * 0.5,
                            configuration.height as f32 * 0.5,
                            -(configuration.height as f32) * 0.5,
                        ],
                        framework,
                    )),
                ]),
                buffer_type: framework::BufferType::Uniform,
                allow_write: false,
                allow_read: false,
            },
        );
        let camera_bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("BitmapLayer camera bind layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let camaera_bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("BitmapLayer Camera bind group"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(camera.binding_resource()),
                }],
            });
        Self {
            texture,
            configuration,
            camera_buffer: camera,
            camaera_bind_group,
        }
    }

    pub fn texture(&self) -> &Texture2d {
        &self.texture
    }

    pub fn bind_group(&self) -> &BindGroup {
        self.texture.bind_group()
    }

    pub fn size(&self) -> Vector2<f32> {
        Vector2 {
            x: self.configuration.width as f32,
            y: self.configuration.height as f32,
        }
    }

    pub fn camera_bind_group(&self) -> &BindGroup {
        &self.camaera_bind_group
    }
}
