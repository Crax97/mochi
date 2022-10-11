use std::{
    cell::RefMut,
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use uuid::Uuid;

pub(crate) type AssetMap<T> = Arc<RwLock<HashMap<Uuid, T>>>;
pub struct AssetRef<'a, T> {
    pub(crate) in_ref: RwLockReadGuard<'a, HashMap<Uuid, T>>,
    pub(crate) id: AssetId,
}

impl<'a, T> Deref for AssetRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.in_ref.get(&self.id.0).unwrap()
    }
}

pub struct AssetRefMut<'a, T> {
    pub(crate) in_ref: RwLockWriteGuard<'a, HashMap<Uuid, T>>,
    pub(crate) id: AssetId,
}

impl<'a, T> Deref for AssetRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.in_ref.get(&self.id.0).unwrap()
    }
}

impl<'a, T> DerefMut for AssetRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.in_ref.get_mut(&self.id.0).unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct AssetId(pub(crate) Uuid);

impl AssetId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Debug for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Asset ID").field(&self.0).finish()
    }
}
