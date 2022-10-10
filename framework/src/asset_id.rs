use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use uuid::Uuid;

pub struct Asset<T>(Arc<T>);

impl<T> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for Asset<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T> DerefMut for Asset<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.0).expect("This arc is not unique!")
    }
}

pub(crate) struct AllocatedAsset<T> {
    pub(crate) asset: Asset<T>,
    pub(crate) refcount: u32,
}

impl<T> AllocatedAsset<T> {
    pub(crate) fn new(raw_asset: T) -> Self {
        Self {
            asset: Asset(Arc::new(raw_asset)),
            refcount: 1,
        }
    }
}

pub(crate) type AssetMap<T> = Arc<Mutex<HashMap<Uuid, AllocatedAsset<T>>>>;

pub struct AssetId<T>(pub(crate) Uuid, pub(crate) AssetMap<T>);

impl<T> AssetId<T> {
    pub(crate) fn new(asset_map: AssetMap<T>) -> Self {
        Self(Uuid::new_v4(), asset_map)
    }
}

impl<T> Clone for AssetId<T> {
    fn clone(&self) -> Self {
        {
            let mut textures = self.1.lock().unwrap();
            textures.get_mut(&self.0).unwrap().refcount += 1;
        }
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T> Drop for AssetId<T> {
    fn drop(&mut self) {
        let mut textures = self.1.lock().unwrap();
        let refcount = {
            let texture_slot = textures.get_mut(&self.0).unwrap();
            texture_slot.refcount -= 1;
            texture_slot.refcount
        };
        if refcount == 0 {
            textures.remove(&self.0).unwrap();
        }
    }
}

impl<T> std::fmt::Debug for AssetId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Asset ID").field(&self.0).finish()
    }
}
