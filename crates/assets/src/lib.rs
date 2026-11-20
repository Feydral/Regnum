mod manager;

pub use manager::AssetManager;

use std::{fmt, marker::PhantomData};

pub struct Handle<T> {
    index: u32,
    generation: u32,
    marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            marker: PhantomData,
        }
    }

    pub fn index(self) -> usize {
        self.index as usize
    }

    pub fn generation(self) -> u32 {
        self.generation
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Handle<T> {}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Handle({}v{})", self.index, self.generation)
    }
}

pub enum AssetEvent<T> {
    Changed(Handle<T>),
    Removed(Handle<T>),
}

struct Slot<T> {
    generation: u32,
    value: Option<T>,
    dirty: bool,
}

pub struct Assets<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    events: Vec<AssetEvent<T>>,
}

impl<T> Default for Assets<T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl<T> Assets<T> {
    pub fn add(&mut self, value: T) -> Handle<T> {
        let index = match self.free.pop() {
            Some(index) => index,
            None => {
                self.slots.push(Slot {
                    generation: 0,
                    value: None,
                    dirty: false,
                });
                (self.slots.len() - 1) as u32
            }
        };

        let slot = &mut self.slots[index as usize];
        slot.value = Some(value);
        slot.dirty = true;

        let handle = Handle::new(index, slot.generation);
        self.events.push(AssetEvent::Changed(handle));
        handle
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let slot = self.slots.get(handle.index())?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index())?;
        if slot.generation != handle.generation || slot.value.is_none() {
            return None;
        }
        if !slot.dirty {
            slot.dirty = true;
            self.events.push(AssetEvent::Changed(handle));
        }
        slot.value.as_mut()
    }

    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let slot = self.slots.get_mut(handle.index())?;
        if slot.generation != handle.generation {
            return None;
        }
        let value = slot.value.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(handle.index);
        self.events.push(AssetEvent::Removed(handle));
        Some(value)
    }

    pub fn take_events(&mut self) -> Vec<AssetEvent<T>> {
        let events = std::mem::take(&mut self.events);
        for event in &events {
            if let AssetEvent::Changed(handle) = event
                && let Some(slot) = self.slots.get_mut(handle.index())
            {
                slot.dirty = false;
            }
        }
        events
    }
}
