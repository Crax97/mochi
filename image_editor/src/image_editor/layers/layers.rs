use std::cell::RefCell;

use cgmath::{point2, point3, vec2, ElementWise, Point2, Rad, Vector2};
use framework::framework::TextureId;
use framework::renderer::renderer::Renderer;
use framework::scene::Transform2d;
use framework::{Box2d, Framework, RgbaTexture2D, Texture, TextureConfiguration, TextureUsage};
use uuid::Uuid;

use crate::blend_settings::BlendMode;

use super::ChunkedLayer;

#[derive(Clone, PartialEq)]
pub struct LayerSettings {
    pub name: String,
    pub blend_mode: BlendMode,
    pub is_enabled: bool,
    pub is_locked: bool,
    pub is_mask: bool,
    pub opacity: f32,
}

impl LayerSettings {
    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
            blend_mode: BlendMode::Normal,
            is_enabled: true,
            is_locked: false,
            is_mask: false,
            opacity: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct ShaderLayerSettings {
    pub opacity: f32,
}

impl LayerId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

pub trait LayerBase {
    fn id(&self) -> &LayerId;
}

pub struct Layer {
    id: LayerId,
    transform: Transform2d,
    settings: LayerSettings,

    pub layer_type: LayerType,
    needs_settings_update: RefCell<bool>,
    needs_bitmap_update: RefCell<bool>,
}

impl LayerBase for Layer {
    fn id(&self) -> &LayerId {
        &self.id
    }
}

pub struct LayerCreationInfo {
    pub name: String,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct LayerId(Uuid);

#[derive(Debug)]
pub enum LayerType {
    Image {
        texture: TextureId,
        dimensions: Vector2<u32>,
    },
    Chonky(ChunkedLayer),
    Group, // This is just a marker type
}

#[derive(Clone, Copy, Debug)]
pub enum OperationResult {
    Rerender,
    Update,
    RenderAndUpdate,
    None,
}

pub trait LayerOperation {
    fn accept(&self, layer: &Layer) -> bool;
    fn execute(
        &self,
        layer: &mut Layer,
        bounds: Box2d,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> OperationResult;
}

pub trait ImageLayerOperation {
    fn image_op(
        &self,
        image_texture: &TextureId,
        dimensions: &Vector2<u32>,
        owning_layer: &Layer,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> OperationResult;
}

impl<T: ImageLayerOperation> LayerOperation for T {
    fn accept(&self, layer: &Layer) -> bool {
        match &layer.layer_type {
            LayerType::Image { .. } => true,
            _ => false,
        }
    }

    fn execute(
        &self,
        layer: &mut Layer,
        _: Box2d,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> OperationResult {
        match &layer.layer_type {
            LayerType::Image {
                texture,
                dimensions,
            } => self.image_op(texture, dimensions, layer, renderer, framework),
            _ => unreachable!(),
        }
    }
}

pub trait ChunkedLayerOperation {
    fn chunk_op(
        &self,
        chunk: &TextureId,
        index: &Point2<i64>,
        chunk_position: &Point2<f32>,
        owning_layer: &Layer,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> OperationResult;
}

impl Layer {
    pub fn new_image(
        image: RgbaTexture2D,
        creation_info: LayerCreationInfo,
        framework: &mut Framework,
    ) -> Self {
        let (w, h) = (image.width(), image.height());
        let texture = framework.allocate_texture2d(
            image,
            TextureConfiguration {
                label: Some(format!("Layer \"{}\" texture", creation_info.name).as_str()),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
        Self {
            id: LayerId::new(),

            layer_type: LayerType::Image {
                texture,
                dimensions: vec2(w, h),
            },
            settings: LayerSettings {
                name: creation_info.name,
                blend_mode: BlendMode::Normal,
                is_enabled: true,
                is_locked: false,
                is_mask: false,
                opacity: 1.0,
            },

            needs_settings_update: RefCell::new(true),
            needs_bitmap_update: RefCell::new(true),
            transform: Transform2d {
                position: point3(creation_info.position.x, creation_info.position.y, 0.0),
                scale: creation_info.scale,
                rotation_radians: Rad(creation_info.rotation_radians),
            },
        }
    }

    pub fn new_chonky(creation_info: LayerCreationInfo) -> Self {
        static CHUNK_SIZE: u32 = 256;
        Self {
            id: LayerId::new(),
            transform: Transform2d {
                position: point3(creation_info.position.x, creation_info.position.y, 0.0),
                scale: creation_info.scale,
                rotation_radians: Rad(creation_info.rotation_radians),
            },
            settings: LayerSettings::new(&creation_info.name),
            layer_type: LayerType::Chonky(ChunkedLayer::new(&creation_info.name, CHUNK_SIZE)),
            needs_settings_update: RefCell::new(false),
            needs_bitmap_update: RefCell::new(false),
        }
    }

    pub fn needs_settings_update(&self) -> bool {
        let ret = self.needs_settings_update.borrow().clone();
        *self.needs_settings_update.borrow_mut() = false;
        ret
    }

    pub fn needs_bitmap_update(&self) -> bool {
        let ret = self.needs_bitmap_update.borrow().clone();
        *self.needs_bitmap_update.borrow_mut() = false;
        ret
    }

    pub fn execute_operation<O: LayerOperation>(
        &mut self,
        op: O,
        bounds: Box2d,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) {
        if op.accept(&self) {
            match op.execute(self, bounds, renderer, framework) {
                OperationResult::Rerender => *self.needs_bitmap_update.borrow_mut() = true,
                OperationResult::Update => *self.needs_settings_update.borrow_mut() = true,
                OperationResult::RenderAndUpdate => {
                    *self.needs_bitmap_update.borrow_mut() = true;
                    *self.needs_settings_update.borrow_mut() = true;
                }
                OperationResult::None => {}
            }
        }
    }

    pub fn mark_dirty(&mut self) {
        *self.needs_bitmap_update.borrow_mut() = true;
    }

    pub fn translate(&mut self, delta: Vector2<f32>) {
        self.transform.translate(delta.extend(0.0));
        self.mark_dirty();
    }

    pub fn replace_texture(&mut self, new_texture: TextureId) {
        match &mut self.layer_type {
            LayerType::Image { texture, .. } => *texture = new_texture,
            _ => unreachable!(),
        };
        self.mark_dirty();
    }

    pub fn settings(&self) -> &LayerSettings {
        &self.settings
    }

    pub fn set_settings(&mut self, new_settings: LayerSettings) {
        self.settings = new_settings;
        *self.needs_settings_update.borrow_mut() = true;
        self.mark_dirty();
    }

    pub fn transform(&self) -> Transform2d {
        self.transform
    }

    pub fn pixel_transform(&self) -> Transform2d {
        let bounds = self.bounds();
        Transform2d {
            position: point3(bounds.center.x, bounds.center.x, 0.0),
            scale: self.bounds().extents,
            rotation_radians: self.transform.rotation_radians,
        }
    }

    pub fn bounds(&self) -> Box2d {
        match &self.layer_type {
            LayerType::Image { dimensions, .. } => Box2d {
                center: point2(self.transform.position.x, self.transform.position.y),
                extents: dimensions.cast::<f32>().unwrap().mul_element_wise(0.5),
            },
            LayerType::Chonky(map) => map.bounds(),
            LayerType::Group => unreachable!(),
        }
    }

    pub fn id(&self) -> &LayerId {
        &self.id
    }

    pub(crate) fn size(&self) -> Vector2<u32> {
        self.bounds().extents.cast::<u32>().unwrap() * 2
    }
}
