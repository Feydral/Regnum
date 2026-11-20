use std::{fmt, marker::PhantomData};

pub struct Handle<T> {
    pub(crate) index: u32,
    pub(crate) generation: u32,
    marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    pub(crate) fn new(index: u32, generation: u32) -> Self {
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
