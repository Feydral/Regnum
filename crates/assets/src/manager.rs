use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{Assets, Handle};

#[derive(Default)]
pub struct AssetManager {
    storages: HashMap<TypeId, Box<dyn Any>>,
}

impl AssetManager {
    pub fn add<T: 'static>(&mut self, asset: T) -> Handle<T> {
        self.storage_mut::<T>().add(asset)
    }

    pub fn get<T: 'static>(&self, handle: Handle<T>) -> Option<&T> {
        self.storages
            .get(&TypeId::of::<T>())?
            .downcast_ref::<Assets<T>>()?
            .get(handle)
    }

    pub fn get_mut<T: 'static>(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.storage_mut::<T>().get_mut(handle)
    }

    pub fn remove<T: 'static>(&mut self, handle: Handle<T>) -> Option<T> {
        self.storage_mut::<T>().remove(handle)
    }

    pub fn storage_mut<T: 'static>(&mut self) -> &mut Assets<T> {
        self.storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Assets::<T>::default()))
            .downcast_mut::<Assets<T>>()
            .expect("storage type matches its TypeId")
    }
}
