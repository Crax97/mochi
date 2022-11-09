use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

use crossbeam_channel::{Receiver, Sender};
use uuid::Uuid;

pub(crate) enum RefEvent<T> {
    IncrementRef(T),
    DecrementRef(T),
}

pub(crate) struct RefCounted<T> {
    pub(crate) value: T,
    refs: AtomicU32,
}

impl<T> RefCounted<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            refs: AtomicU32::new(1),
        }
    }
}

impl<T: Debug> Debug for RefCounted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefCounted")
            .field("value", &self.value)
            .field("refs", &self.refs)
            .finish()
    }
}

pub(crate) struct AssetMap<T> {
    pub(crate) map: HashMap<Uuid, RefCounted<T>>,
    event_receiver: Receiver<RefEvent<Uuid>>,
    event_sender: Sender<RefEvent<Uuid>>,
    taken_this_update: Vec<Uuid>,
}

pub struct AssetId<T> {
    pub(crate) index: Uuid,
    pub(crate) event_sender: Sender<RefEvent<Uuid>>,
    pub(crate) phantom: PhantomData<T>,
}

impl<T> Clone for AssetId<T> {
    fn clone(&self) -> Self {
        self.event_sender
            .send(RefEvent::IncrementRef(self.index.clone()))
            .expect("Failure when sending ref increment!");
        Self {
            index: self.index.clone(),
            event_sender: self.event_sender.clone(),
            phantom: self.phantom.clone(),
        }
    }
}
impl<T> Drop for AssetId<T> {
    fn drop(&mut self) {
        self.event_sender
            .send(RefEvent::DecrementRef(self.index.clone()))
            .expect("Failure when sending def increment!");
    }
}

impl<T> AssetMap<T> {
    pub(crate) fn new() -> Self {
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();
        Self {
            map: HashMap::new(),
            event_receiver,
            event_sender,
            taken_this_update: vec![],
        }
    }

    pub(crate) fn insert(&mut self, value: T) -> AssetId<T> {
        let id = RefCounted::new(value);
        let uuid = Uuid::new_v4();
        self.map.insert(uuid.clone(), id);
        let id = AssetId {
            index: uuid,
            event_sender: self.event_sender.clone(),
            phantom: PhantomData::<T>,
        };
        id
    }

    pub(crate) fn get(&self, id: &AssetId<T>) -> &T {
        &self
            .map
            .get(&id.index)
            .unwrap_or_else(|| panic!("No asset with id {:?}", id))
            .value
    }

    pub(crate) fn get_mut(&mut self, id: &AssetId<T>) -> &mut T {
        &mut self
            .map
            .get_mut(&id.index)
            .unwrap_or_else(|| panic!("No asset with id {:?}", id))
            .value
    }
    pub(crate) fn update(&mut self) {
        while let Ok(update) = self.event_receiver.try_recv() {
            match update {
                RefEvent::IncrementRef(index) => self.increment_ref(index),
                RefEvent::DecrementRef(index) => self.decremente_ref(index),
            }
        }
        self.taken_this_update.clear();
    }

    fn increment_ref(&mut self, index: Uuid) {
        let asset = self
            .map
            .get_mut(&index)
            .expect("Asset not stored in map! Something broke badly");
        asset.refs.fetch_add(1, Ordering::Relaxed);
    }

    fn decremente_ref(&mut self, index: Uuid) {
        if self.taken_this_update.contains(&index) {
            return;
        }
        let refs_before_sub = {
            let asset = self
                .map
                .get_mut(&index)
                .expect("Asset not stored in map! Something broke badly");
            asset.refs.fetch_sub(1, Ordering::Relaxed)
        };
        if refs_before_sub == 1 {
            self.map.remove(&index);
        }
    }

    pub(crate) fn take(&mut self, view: AssetId<T>) -> T {
        let uuid = view.index.clone();
        drop(view);
        let asset = self.map.remove(&uuid).unwrap();
        if asset.refs.load(Ordering::Relaxed) != 1 {
            panic!("AssetMap::take is only allowed for textures that have one reference");
        }
        self.taken_this_update.push(uuid);
        self.update();
        asset.value
    }
}

impl<T: Debug> Debug for AssetMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerAssetMap")
            .field("map", &self.map)
            .finish()
    }
}

impl<T> std::fmt::Debug for AssetId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Asset ID").field(&self.index).finish()
    }
}
